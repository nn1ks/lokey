use bt_hci::event::Vendor;
use bt_hci::param::{LeAdvReportsIter, LeExtAdvReportsIter};
use core::cell::RefCell;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use trouble_host::prelude::EventHandler;
use trouble_host::{BleHostError, Controller, PacketPool};

pub struct Runner<'r, 'e, C: Controller, P: PacketPool> {
    ble_stack_runner: trouble_host::prelude::Runner<'r, C, P>,
    event_handler: &'e RunnerEventHandler,
}

impl<'r, 'e, C: Controller, P: PacketPool> Runner<'r, 'e, C, P> {
    pub fn new(
        ble_stack_runner: trouble_host::prelude::Runner<'r, C, P>,
        event_handler: &'e RunnerEventHandler,
    ) -> Self {
        Self {
            ble_stack_runner,
            event_handler,
        }
    }

    pub async fn run(&mut self) -> Result<(), BleHostError<C::Error>> {
        self.ble_stack_runner
            .run_with_handler(self.event_handler)
            .await
    }
}

pub struct RunnerEventHandler {
    inner: Mutex<CriticalSectionRawMutex, RefCell<Option<&'static (dyn EventHandler + Sync)>>>,
}

impl RunnerEventHandler {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(None)),
        }
    }

    fn lock_and_then<F>(&self, f: F)
    where
        F: FnOnce(&'static dyn EventHandler),
    {
        self.inner.lock(|event_handler| {
            let event_handler = event_handler.borrow();
            if let Some(event_handler) = &*event_handler {
                f(*event_handler)
            }
        })
    }

    pub fn register<H: EventHandler + Sync>(&self, event_handler: &'static H) -> bool {
        self.inner.lock(|current_event_handler| {
            let mut current_event_handler = current_event_handler.borrow_mut();
            if current_event_handler.is_some() {
                return false;
            }
            *current_event_handler = Some(event_handler);
            true
        })
    }
}

impl Default for RunnerEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler for RunnerEventHandler {
    fn on_vendor(&self, vendor: &Vendor) {
        self.lock_and_then(|event_handler| event_handler.on_vendor(vendor));
    }

    fn on_adv_reports(&self, reports: LeAdvReportsIter) {
        self.lock_and_then(|event_handler| event_handler.on_adv_reports(reports));
    }

    fn on_ext_adv_reports(&self, reports: LeExtAdvReportsIter) {
        self.lock_and_then(|event_handler| event_handler.on_ext_adv_reports(reports));
    }

    fn on_packets_completed(&self, num_completed: usize) {
        self.lock_and_then(|event_handler| event_handler.on_packets_completed(num_completed));
    }
}
