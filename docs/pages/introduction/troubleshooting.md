# Troubleshooting

## Key or mouse events are not being sent

If the device is not sending key or mouse events to the host, one of the following may be the cause:

- Incorrect state type in `main` context

    Ensure the `Context` type passed to `main` uses the correct state type.

    ```rust{2}
    #[lokey::device]
    async fn main(context: Context<..., ..., MyState>) {
        ...
    }
    ```

- Missing keyboard or mouse report state

    Ensure your state type includes `KeyboardReportState` and/or `MouseReportState`.

    ```rust{3,4}
    #[derive(Default, State)]
    struct MyState {
        keyboard_report: KeyboardReportState,
        mouse_report: MouseReportState,
        // ...
    }
    ```

- Missing layer manager

    If you are using the `PerLayer` action or the `layout!` macro, ensure your state type includes `LayerManager` and the field has the `#[state(query)]` attribute.

    ```rust{3,4}
    #[derive(Default, State)]
    struct MyState {
        #[state(query)]
        layer_manager: LayerManager<0>,
        // ...
    }
    ```

## Firmware crashes after startup

If the device firmware panics shortly after startup, one of the following may be the cause:

- Resource limits are not configured correctly

    The configured resource limits may be too low for your firmware. Additionally, if resources are not configured at all, they default to 0, which causes a panic as soon as one of those resources is used.

    Use the following features to configure resource limits:

    - `max-internal-message-size-*`
    - `internal-receiver-slots-*`
    - `external-receiver-slots-*`
    - `external-observer-slots-*`

    See the API documentation of [`lokey`](https://docs.rs/lokey) for more information.
