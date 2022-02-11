/*
 * A rust replacement for vsv
 *
 * Original: https://github.com/bahamas10/vsv
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 25, 2022
 * License: MIT
 */

use std::env;

use anyhow::{anyhow, Context, Result};
use clap::crate_name;
use rayon::prelude::*;
use yansi::{Color, Paint, Style};

mod arguments;
mod config;
mod die;
mod runit;
mod service;
mod utils;

use arguments::Commands;
use config::Config;
use die::die;
use service::Service;

macro_rules! verbose {
    ($cfg:expr, $fmt:expr $(, $args:expr )* $(,)? ) => {
        if $cfg.verbose > 0 {
            let s = format!($fmt $(, $args)*);
            eprintln!(">  {}", Style::default().dimmed().paint(s));
        }
    };
}

fn do_external(cfg: &Config, args: &[String]) -> Result<()> {
    assert!(!args.is_empty());

    let sv = config::SV_PROG.to_owned();

    if args.len() < 2 {
        return Err(anyhow!("argument expected for '{} {}'", sv, args[0]));
    }

    // format arguments
    let args_s = args.join(" ");

    // set SVDIR env to match what user wanted
    env::set_var(config::ENV_SVDIR, &cfg.svdir);

    println!(
        "[{}] {}",
        crate_name!(),
        Color::Cyan.paint(format!(
            "Running {} command ({}={:?} {} {})",
            sv,
            config::ENV_SVDIR,
            &cfg.svdir,
            sv,
            &args_s
        ))
    );

    // run the actual program
    let status = utils::run_program_get_status(&sv, args);

    // check the process status
    match status {
        Ok(status) => {
            let code = status.code().unwrap_or(-1);
            let color = match code {
                0 => Color::Green,
                _ => Color::Red,
            };

            // print exit code
            println!(
                "[{}] {}",
                crate_name!(),
                color.paint(format!("[{} {}] exit code {}", sv, &args_s, code))
            );

            match code {
                0 => Ok(()),
                _ => Err(anyhow!("call to {} failed", sv)),
            }
        }
        Err(err) => Err(anyhow!("failed to execute {}: {}", sv, err)),
    }
}

fn do_status(cfg: &Config) -> Result<()> {
    // find all services
    let services = runit::get_services(&cfg.svdir, cfg.log, cfg.get_filter())
        .with_context(|| {
        format!("failed to list services in {:?}", cfg.svdir)
    })?;

    // loop each service found (just gather data here, can be done in parallel)
    let services: Vec<(Service, Vec<String>)> = services
        .par_iter()
        .map(|service| Service::from_runit_service(service, cfg.tree))
        .collect();

    // print gathared data
    let bold_style = Style::default().bold();

    println!();
    verbose!(cfg, "found {} services in {:?}", services.len(), cfg.svdir);
    println!(
        "{}",
        utils::format_status_line(
            ("", &bold_style),
            ("SERVICE", &bold_style),
            ("STATE", &bold_style),
            ("ENABLED", &bold_style),
            ("PID", &bold_style),
            ("COMMAND", &bold_style),
            ("TIME", &bold_style),
        )
    );

    // print each service found
    for (service, messages) in services {
        println!("{}", service);

        // print pstree if applicable
        if cfg.tree {
            let tree_s = service.format_pstree();
            println!("{}", tree_s);
        }

        // print any verbose messages/warnings generated by the service
        for message in messages {
            verbose!(cfg, "{}", message);
        }
    }

    if !cfg.tree {
        println!();
    }

    Ok(())
}

fn do_main() -> Result<()> {
    // disable color until we absolutely know we want it
    Paint::disable();

    // parse CLI options + env vars
    let args = arguments::parse();
    let cfg = Config::from_args(&args)?;

    if cfg.colorize {
        Paint::enable();
    }

    // figure out subcommand to run
    match &args.command {
        Some(Commands::External(args)) => do_external(&cfg, args),
        None | Some(Commands::Status { .. }) => do_status(&cfg),
    }
}

fn main() {
    let ret = do_main();

    if let Err(err) = ret {
        die!(1, "{}: {:?}", Color::Red.paint("error"), err);
    }
}
