use scenario::{Inactive, Scenario};

#[cfg(test)]
mod cli_process;
pub mod scenario;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    env_logger::init();
}

impl Default for Scenario<Inactive> {
    fn default() -> Self {
        Scenario::<Inactive>::new(
            env!("CARGO_BIN_EXE_pulso"),
            std::time::Duration::new(1, 500),
        )
    }
}
