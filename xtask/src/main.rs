use structopt::StructOpt;

#[derive(StructOpt)]
enum Command {
    Run {
        #[structopt(long)]
        _debug: bool,
    },
    Build {
        #[structopt(long)]
        _debug: bool
    }
}

fn build_kernel() -> anyhow::Result<()> {
    let _dir = xshell::pushd("./lsd");
    xshell::cmd!("cargo build --release").run()?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Command::from_args();

    match args {
        Command::Build { _debug } => {
            build_kernel()?;
        },
        Command::Run { _debug } => {
            build_kernel()?;

            let debug_log: &[&str] = match true {
                //true => &["-D", "debug.log", "-d", "int,guest_errors"],
                true => &["-d", "int,guest_errors"],
                false => &[],
            };

            #[rustfmt::skip]
            xshell::cmd!("
                qemu-system-riscv64
                    -machine virt
                    -cpu rv64
                    -smp 1
                    -m 8G
                    -global virtio-mmio.force-legacy=false
                    -object rng-random,filename=/dev/urandom,id=rng0 
                    -device virtio-rng-device,rng=rng0 
                    -device virtio-gpu-device
                    -bios opensbi-riscv64-generic-fw_jump.bin
                    -kernel lsd/target/riscv64gc-unknown-none-elf/release/lsd
                    -serial mon:stdio
                    -no-reboot
                    {debug_log...}
            ").run()?;
        }
    }

    Ok(())
}
