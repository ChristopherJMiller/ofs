
import yargs from "https://deno.land/x/yargs@v17.0.1-deno/deno.ts"
import { Arguments } from "https://deno.land/x/yargs@v17.0.1-deno/deno-types.ts"
import { dirname, join, fromFileUrl } from "https://deno.land/std@0.100.0/path/mod.ts";
import Spinner from 'https://deno.land/x/cli_spinners@v0.0.2/mod.ts';

type ProjectDir = string;

const getProjectDir = (project: string): ProjectDir => {
  let scriptDir = dirname(import.meta.url)
  let joinedPath = join(scriptDir, '..', project)
  return fromFileUrl(joinedPath) as ProjectDir
}

const getCargoFile = (path: ProjectDir): string => join(path, 'Cargo.toml')
const getTargetFile = (path: string, target: string): string => join(path, target)
const getArduinoElfLocation = (path: string, elfName: string): string => join(path, 'target', 'avr-atmega328p', 'release', elfName)

const buildController = async (): Promise<number> => {
  let projectDir = getProjectDir('controller')
  let cargoFile = getCargoFile(projectDir)
  let target = getTargetFile(projectDir, 'avr-atmega328p.json')
  const buildc = Deno.run({
    cmd: [
      "cargo",
      "build",
      "--release",
      `--manifest-path=${cargoFile}`
    ],
    env: {
      CARGO_BUILD_TARGET: target,
      CARGO_UNSTABLE_BUILD_STD: 'core'
    }
  })
  
  const { code } = await buildc.status()

  return code
}

const handleBuildCommand = async () => {
  console.log('Building Controller...')
  let code = await buildController()
  Deno.exit(code)
}

const handleFlashCommand = async () => {
  const spinner = Spinner.getInstance()

  spinner.start('Building Library...')
  let buildCode = await buildController()
  if (buildCode !== 0) {
    console.error('Build failed, cannot flash!')
    Deno.exit(buildCode)
  }

  let projectDir = getProjectDir('controller')
  let elfPath = getArduinoElfLocation(projectDir, 'ofs-controller.elf')

  spinner.setText('Flashing Arduino...')
  const flashc = Deno.run({
    cmd: [
      "avrdude",
      "-q",
      "-patmega328p",
      "-carduino",
      '-P/dev/ttyACM0',
      '-D',
      `-Uflash:w:${elfPath}:e`
    ],
    stdout: "piped",
    stderr: "piped",
  })

  const { code } = await flashc.status()

  // Reading the outputs closes their pipes
  const rawOutput = await flashc.output()
  const rawError = await flashc.stderrOutput()

  if (code == 0) {
    spinner.succeed('Flashing Complete!')
    await Deno.stdout.write(rawOutput)
    console.log('') // New Line
  } else {
    const errorString = new TextDecoder().decode(rawError)
    console.log(errorString)
    spinner.fail('Flashing Failed!')
    console.log('')
  }

  Deno.exit(code)
}

yargs(Deno.args)
  .usage("Usage: ofs.ts <command>")
  .command("buildc", "build controller library", handleBuildCommand)
  .command("flashc", "flash controller", handleFlashCommand)
  .strictCommands()
  .demandCommand(1)
  .parse()
