use structopt::StructOpt;

#[derive(StructOpt)]
enum Command {
    Run {
        #[structopt(long)]
        debug: bool,
    },
}

fn build_kernel() -> anyhow::Result<()> {
    let _dir = xshell::pushd("./lsd");
    xshell::cmd!("cargo build --release").run()?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Command::from_args();

    match args {
        Command::Run { debug } => {
            build_kernel()?;

            let debug_log: &[&str] = match true {
                true => &["-D", "debug.log", "-d", "int,guest_errors"],
                false => &[],
            };

            #[rustfmt::skip]
            xshell::cmd!("
                qemu-system-riscv64
                    -machine virt
                    -cpu rv64
                    -smp 8
                    -m 128M
                    -bios opensbi-riscv64-generic-fw_jump.bin
                    -kernel lsd/target/riscv64gc-unknown-none-elf/release/lsd
                    -serial mon:stdio
                    -nographic
                    {debug_log...}
            ").run()?;
        }
    }

    Ok(())
}
