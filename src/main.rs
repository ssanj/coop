use args::cli;
use workflow::CoopWorkflow;

mod args;
mod model;
mod progress;
mod monitor;
mod copy;
mod workflow;

fn main() {
  let args = cli::get_cli_args();
  let workflow = CoopWorkflow::new(args);
  ()
}
