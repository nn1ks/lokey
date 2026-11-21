#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));

const defaultOutputs = {
  dot: "architecture-graph.dot",
  png: "architecture-graph.png",
};

const defaultCargoArgs = ["--build-deps", "--workspace-only", "--all-features"];

const run = (command, args) => {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  });

  if (result.error) {
    throw new Error(`Failed to run ${command}: ${result.error.message}`);
  }

  if (result.status !== 0) {
    const stdout = result.stdout?.trim();
    throw new Error(
      [
        `Command failed (${result.status}): ${command} ${args.join(" ")}`,
        stdout,
      ]
        .filter(Boolean)
        .join("\n"),
    );
  }

  return result.stdout ?? "";
};

const parseDepgraphDot = (dot) => {
  const nodes = new Map();
  const edges = [];

  for (const line of dot.split(/\r?\n/)) {
    const trimmed = line.trim();

    if (
      !trimmed ||
      trimmed.startsWith("digraph") ||
      trimmed === "}" ||
      trimmed === "};"
    ) {
      continue;
    }

    const nodeMatch = trimmed.match(/^(\d+)\s*\[(.*)\]\s*;?\s*$/);
    if (nodeMatch) {
      const [, id, attributes] = nodeMatch;
      const labelMatch = attributes.match(/label\s*=\s*"([^"]*)"/);

      if (!labelMatch) {
        continue;
      }

      nodes.set(id, {
        id,
        label: labelMatch[1],
        isMacro: /\bcolor\s*=\s*green3\b/.test(attributes),
      });
      continue;
    }

    const edgeMatch = trimmed.match(/^(\d+)\s*->\s*(\d+)\s*\[(.*)\]\s*;?\s*$/);
    if (edgeMatch) {
      const [, from, to, attributes] = edgeMatch;

      edges.push({
        from,
        to,
        dotted: /\bstyle\s*=\s*dotted\b/.test(attributes),
      });
    }
  }

  return { nodes, edges };
};

const quote = (value) => JSON.stringify(value);

const escapeHtml = (value) =>
  value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");

const validateConfig = (config, nodesByLabel) => {
  if (!config.groups || typeof config.groups !== "object") {
    throw new Error("Config must define a groups object.");
  }

  if (!Array.isArray(config.rows)) {
    throw new Error("Config must define a rows array.");
  }

  if (!Array.isArray(config.layoutEdges)) {
    throw new Error("Config must define a layoutEdges array.");
  }

  const groupNames = Object.keys(config.groups);
  const crateToGroup = new Map();

  for (const [groupName, crates] of Object.entries(config.groups)) {
    if (!Array.isArray(crates) || crates.length === 0) {
      throw new Error(`Group "${groupName}" must contain at least one crate.`);
    }

    for (const crate of crates) {
      const existingGroup = crateToGroup.get(crate);

      if (existingGroup === groupName) {
        throw new Error(
          `Crate "${crate}" appears more than once in group "${groupName}".`,
        );
      }

      if (existingGroup) {
        throw new Error(
          `Crate "${crate}" appears in more than one group: "${existingGroup}" and "${groupName}".`,
        );
      }

      if (!nodesByLabel.has(crate)) {
        throw new Error(
          `Group "${groupName}" references crate "${crate}", which is not in the cargo depgraph output.`,
        );
      }

      crateToGroup.set(crate, groupName);
    }
  }

  const ungroupedCrates = [...nodesByLabel.keys()].filter(
    (crate) => !crateToGroup.has(crate),
  );

  for (const groupName of groupNames) {
    if (ungroupedCrates.includes(groupName)) {
      throw new Error(
        `Group name "${groupName}" collides with an ungrouped crate name.`,
      );
    }
  }

  const entities = new Set([...groupNames, ...ungroupedCrates]);
  const rowEntities = new Set();

  for (const row of config.rows) {
    if (!Array.isArray(row) || row.length === 0) {
      throw new Error("Each row must contain at least one entity.");
    }

    for (const entity of row) {
      if (!entities.has(entity)) {
        throw new Error(`Row references unknown entity "${entity}".`);
      }

      if (rowEntities.has(entity)) {
        throw new Error(`Entity "${entity}" appears in more than one row.`);
      }

      rowEntities.add(entity);
    }
  }

  const missingFromRows = [...entities].filter(
    (entity) => !rowEntities.has(entity),
  );

  if (missingFromRows.length > 0) {
    throw new Error(
      `Entities not assigned to a row: ${missingFromRows.join(", ")}`,
    );
  }

  for (const [from, to] of config.layoutEdges) {
    if (!entities.has(from) || !entities.has(to)) {
      throw new Error(
        `Layout edge references unknown entity: ${from} -> ${to}`,
      );
    }
  }

  return { entities, ungroupedCrates };
};

const renderGroupNode = (name, crates, nodesByLabel) => {
  const crateRows = crates
    .map((crate) => {
      const text = escapeHtml(crate);
      const content = nodesByLabel.get(crate)?.isMacro
        ? `<FONT COLOR="green3">${text}</FONT>`
        : text;

      return `                <TR><TD ALIGN="LEFT">${content}</TD></TR>`;
    })
    .join("\n");

  return `    ${quote(name)} [
        label = <
            <TABLE BORDER="0" CELLBORDER="0" CELLSPACING="0" CELLPADDING="2">
                <TR><TD ALIGN="CENTER"><B>${escapeHtml(name)}</B></TD></TR>
                <HR/>
${crateRows}
            </TABLE>
        >
    ];`;
};

const renderArchitectureDot = ({
  config,
  nodesById,
  nodesByLabel,
  edges,
  entityForCrate,
}) => {
  const rows = config.rows ?? [];
  const layoutEdges = config.layoutEdges ?? [];
  const font = config.font ?? "Inter";
  const groupNames = Object.keys(config.groups);
  const groupedCrates = new Set(Object.values(config.groups).flat());

  const ungroupedCrates = [...nodesById.values()]
    .map((node) => node.label)
    .filter((crate) => !groupedCrates.has(crate));

  const aggregatedEdges = new Map();

  for (const edge of edges) {
    const from = nodesById.get(edge.from);
    const to = nodesById.get(edge.to);

    if (!from || !to) {
      continue;
    }

    const source = entityForCrate(from.label);
    const target = entityForCrate(to.label);

    if (source === target) {
      continue;
    }

    const key = `${source}\u0000${target}\u0000${edge.dotted ? "dotted" : "solid"}`;

    if (!aggregatedEdges.has(key)) {
      aggregatedEdges.set(key, { source, target, dotted: edge.dotted });
    }
  }

  const solidEdges = [];
  const dottedEdges = [];

  for (const edge of aggregatedEdges.values()) {
    if (edge.dotted) {
      dottedEdges.push(edge);
    } else {
      solidEdges.push(edge);
    }
  }

  const groupNodes = groupNames
    .map((name) => renderGroupNode(name, config.groups[name], nodesByLabel))
    .join("\n");

  const ungroupedNodes = ungroupedCrates
    .map((crate) => {
      const color = nodesByLabel.get(crate)?.isMacro ? ", color = green3" : "";

      return `    ${quote(crate)} [ label = ${quote(crate)}${color} ];`;
    })
    .join("\n");

  const rankRows = rows
    .filter((row) => row.length > 1)
    .map((row) => `    { rank = same; ${row.map(quote).join("; ")}; }`)
    .join("\n");

  const invisibleEdges = layoutEdges
    .map(
      ([from, to]) =>
        `    ${quote(from)} -> ${quote(to)} [ style = invis, constraint = true ];`,
    )
    .join("\n");

  const renderEdge = (edge) => {
    const attributes = edge.dotted
      ? " [ style = dotted, constraint = false ]"
      : "";

    return `    ${quote(edge.source)} -> ${quote(edge.target)}${attributes};`;
  };

  return `digraph {
    graph [
        rankdir = TB,
        newrank = true,
        splines = spline,
        nodesep = 0.5,
        ranksep = 0.9,
        fontname = ${quote(font)},
        fontsize = 12
    ];
    node [
        shape = box,
        style = "rounded,filled",
        fillcolor = white,
        fontname = ${quote(font)},
        fontsize = 11
    ];
    edge [ fontname = ${quote(font)}, fontsize = 9, constraint = false ];

${groupNodes}

${ungroupedNodes}

${rankRows}

    // Invisible edges that pin the rows
${invisibleEdges}

    // Required dependencies
${solidEdges.map(renderEdge).join("\n")}

    // Optional / feature dependencies
${dottedEdges.map(renderEdge).join("\n")}
}
`;
};

const main = () => {
  const configPath =
    process.argv[2] ?? path.join(scriptDir, "architecture-graph.config.json");
  const config = JSON.parse(readFileSync(configPath, "utf8"));
  const outputs = { ...defaultOutputs, ...config.outputs };
  const cargoArgs = config.cargoDepgraphArgs ?? defaultCargoArgs;

  console.log(`Running cargo depgraph ${cargoArgs.join(" ")}`);
  const depgraphDot = run("cargo", ["depgraph", ...cargoArgs]);

  const { nodes, edges } = parseDepgraphDot(depgraphDot);

  if (nodes.size === 0) {
    throw new Error("cargo depgraph produced no nodes.");
  }

  const nodesByLabel = new Map(
    [...nodes.values()].map((node) => [node.label, node]),
  );

  validateConfig(config, nodesByLabel);

  const crateToGroup = new Map();

  for (const [groupName, crates] of Object.entries(config.groups)) {
    for (const crate of crates) {
      crateToGroup.set(crate, groupName);
    }
  }

  const entityForCrate = (crate) => crateToGroup.get(crate) ?? crate;

  const architectureDot = renderArchitectureDot({
    config,
    nodesById: nodes,
    nodesByLabel,
    edges,
    entityForCrate,
  });

  const dotPath = path.join(scriptDir, outputs.dot);
  writeFileSync(dotPath, architectureDot);
  console.log(`Wrote ${path.relative(process.cwd(), dotPath)}`);

  const pngPath = path.join(scriptDir, outputs.png);
  run("dot", ["-Tpng", dotPath, "-o", pngPath]);
  console.log(`Rendered ${path.relative(process.cwd(), pngPath)}`);
};

try {
  main();
} catch (error) {
  console.error(`error: ${error.message}`);
  process.exit(1);
}
