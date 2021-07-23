
import yargs from "https://deno.land/x/yargs@v17.0.1-deno/deno.ts"
import { Arguments } from 'https://deno.land/x/yargs@v17.0.1-deno/deno-types.ts'
import { dirname, basename, join, fromFileUrl } from "https://deno.land/std@0.100.0/path/mod.ts"
import Spinner from 'https://deno.land/x/cli_spinners@v0.0.2/mod.ts'
import { sleep } from "https://deno.land/x/sleep@v1.2.0/mod.ts"

type ProjectDir = string;

interface MCUMap {
  [index: string]: string
}

const getProjectDir = (project: string): ProjectDir => {
  let scriptDir = dirname(import.meta.url)
  let joinedPath = join(scriptDir, '..', project)
  return fromFileUrl(joinedPath) as ProjectDir
}

const getR3FirmwarePath = (path: ProjectDir): string => join(path, 'firmware', 'Rev3.hex')
const getCargoFile = (path: ProjectDir): string => join(path, 'Cargo.toml')
const getTargetFile = (path: string, target: string): string => join(path, target)
const getArduinoElfLocation = (path: string, elfName: string): string => {
  const mcu = getMCU(basename(path))
  return join(path, 'target', `avr-${mcu}`, 'release', elfName)
}
const getMCU = (path: string): string => {
  const MCUs: MCUMap = {
    'controller': 'atmega328p',
    'usb-firmware': 'atmega8u2'
  }

  return MCUs[path]
}

const buildController = async (path: string): Promise<number> => {
  const projectDir = getProjectDir(path)
  const cargoFile = getCargoFile(projectDir)
  const mcu = getMCU(path)
  const target = getTargetFile(projectDir, `avr-${mcu}.json`)
  const buildc = Deno.run({
    cmd: [
      "cargo",
      "build",
      "--release",
      `--manifest-path=${cargoFile}`
    ],
    env: {
      CARGO_BUILD_TARGET: target,
      CARGO_UNSTABLE_BUILD_STD: 'core,alloc'
    }
  })
  
  const { code } = await buildc.status()

  return code
}

const handleBuildCommand = async (path: string) => {
  console.log('Building...')
  let code = await buildController(path)
  Deno.exit(code)
}

const handleFlashCommand = async (path: string) => {
  const spinner = Spinner.getInstance()

  spinner.start('Building Library...')
  const buildCode = await buildController(path)
  if (buildCode !== 0) {
    console.error('Build failed, cannot flash!')
    Deno.exit(buildCode)
  }

  const projectDir = getProjectDir(path)
  const elfPath = getArduinoElfLocation(projectDir, 'ofs-controller.elf')
  const mcu = getMCU(path)

  spinner.setText('Flashing Arduino...')
  const flashc = Deno.run({
    cmd: [
      "avrdude",
      "-q",
      `-p${mcu}`,
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

const handleDFUFlash = async (flashLibrary: boolean) => {
  const spinner = Spinner.getInstance()
  const path = 'usb-firmware'
  spinner.start('')

  if (flashLibrary) {
    spinner.setText('Building Library...')
    const buildCode = await buildController(path)
    if (buildCode !== 0) {
      console.error('Build failed, cannot flash!')
      Deno.exit(buildCode)
    }
  }

  const projectDir = getProjectDir(path)
  const scriptDir = fromFileUrl(dirname(import.meta.url))
  const hexPath = join(scriptDir, 'tmp', 'firmware.hex')
  const elfPath = getArduinoElfLocation(projectDir, 'ofs-usb-firmware.elf')

  spinner.setText('Waiting for DFU Device, please place Arduino in DFU Programmer Mode...')
  while(true) {
    const lsusb = Deno.run({
      cmd: ["lsusb"],
      stdout: "piped",
      stderr: "piped",
    })
    
    const { code } = await lsusb.status()
    const rawOutput = await lsusb.output()
    const rawError = await lsusb.stderrOutput()

    if (code === 0) {
      const lsusbReturn = new TextDecoder().decode(rawOutput)
      // 03eb:2fef Atmel Corp. atmega16u2 DFU bootloader
      if (lsusbReturn.includes('03eb:2fef')) {
        break
      }
    } else {
      const errorString = new TextDecoder().decode(rawError)
      console.log(errorString)
    }

    await sleep(1)
  }

  let firmwarePath

  if (flashLibrary) {
    const avrcopy = Deno.run({
      cmd: ["avr-objcopy", "-Oihex", elfPath, hexPath],
      stdout: "piped",
      stderr: "piped",
    })

    spinner.setText('Copying ELF into Hex...')
    const avrcopyResult = await avrcopy.status()
    if (avrcopyResult.code !== 0) {
      const errorString = new TextDecoder().decode(await avrcopy.stderrOutput())
      console.log(errorString)
      spinner.fail('Obj Copy Failed!')
      console.log('')
      Deno.exit(avrcopyResult.code)
    }

    firmwarePath = hexPath
  } else {
    firmwarePath = getR3FirmwarePath(scriptDir)
  }

  spinner.setText('Erasing...')
  const erase = Deno.run({
    cmd: ["dfu-programmer", "atmega16u2", "erase"],
    stdout: "piped",
    stderr: "piped",
  })

  const eraseResult = await erase.status()

  if (eraseResult.code !== 0 && eraseResult.code !== 5) {
    const errorString = new TextDecoder().decode(await erase.stderrOutput())
    console.log('')
    console.log(errorString)
    spinner.fail('DFU Erase Failed!')
    console.log('')
    Deno.exit(eraseResult.code)
  }
  
  spinner.setText('Flashing...')
  const flash = Deno.run({
    cmd: ["dfu-programmer", "atmega16u2", "flash", firmwarePath],
    stdout: "piped",
    stderr: "piped",
  })
  
  const { code } = await flash.status()
  const rawOutput = await flash.output()
  const rawError = await flash.stderrOutput()

  if (code == 0) {
    spinner.succeed('Flashing Complete! Unplug and replug your arduino for the updated firmware.')
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

const generateSliceNameString = async (str: string) => {
  const byteArray = Array(str.length).fill(0).map((_, i) => str.charCodeAt(i))
  const strArray = byteArray.map((b) => `0x${b.toString(16)}, 0x00`)
  const descriptorSize = strArray.length * 2 + 2
  let strBuild = `[${descriptorSize}, 3, ${strArray.join(',')}]`

  console.log(strBuild)
  Deno.exit(0)
}

yargs(Deno.args)
  .usage("Usage: ofs.ts <command>")
  .command("buildc", "build controller library", () => handleBuildCommand('controller'))
  .command("flashc", "flash controller", () => handleFlashCommand('controller'))
  .command("buildusb", "build usb library", () => handleBuildCommand('usb-firmware'))
  .command("flashusb", "flash usb", () => handleDFUFlash(true))
  .command("restoreusb", "restore arduino back to standard firmware", () => handleDFUFlash(false))
  .command("genstr <string...>", "generate slice for string descriptor", (yargs: any) => yargs.positional('string', {
    describe: 'string to convert'
  }), (argv: Arguments) => generateSliceNameString(argv.string.join(' ')))
  .strictCommands()
  .demandCommand(1)
  .parse()
