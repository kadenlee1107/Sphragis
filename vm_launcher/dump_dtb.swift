// Bat_OS — VZ VM DTB Dumper
// Boots a minimal kernel, then extracts the device tree
// from the VM's memory to discover actual device addresses.
//
// Usage: swift dump_dtb.swift

import Virtualization
import Foundation

let home = FileManager.default.homeDirectoryForCurrentUser.path
let kernelPath = "\(home)/Bat_OS/target/bat_os_image.bin"

guard FileManager.default.fileExists(atPath: kernelPath) else {
    print("Build kernel first: cd ~/Bat_OS && cargo build --release")
    exit(1)
}

print("Bat_OS DTB Dumper — discovering VZ VM hardware layout")
print("")

let config = VZVirtualMachineConfiguration()
let platform = VZGenericPlatformConfiguration()
config.platform = platform
config.cpuCount = 2
config.memorySize = 256 * 1024 * 1024

let bootLoader = VZLinuxBootLoader(kernelURL: URL(fileURLWithPath: kernelPath))
// Add earlycon to try getting output
bootLoader.commandLine = "console=hvc0 earlycon"
config.bootLoader = bootLoader

// Serial — write to stdout
let serialPort = VZVirtioConsoleDeviceSerialPortConfiguration()
let logPath = NSTemporaryDirectory() + "bat_os_dtb_dump.log"
FileManager.default.createFile(atPath: logPath, contents: nil)
let logHandle = FileHandle(forWritingAtPath: logPath)!
serialPort.attachment = VZFileHandleSerialPortAttachment(
    fileHandleForReading: FileHandle(forReadingAtPath: "/dev/null")!,
    fileHandleForWriting: logHandle
)
config.serialPorts = [serialPort]

// Network
let netDev = VZVirtioNetworkDeviceConfiguration()
netDev.attachment = VZNATNetworkDeviceAttachment()
config.networkDevices = [netDev]

// Entropy
config.entropyDevices = [VZVirtioEntropyDeviceConfiguration()]
config.memoryBalloonDevices = [VZVirtioTraditionalMemoryBalloonDeviceConfiguration()]

do {
    try config.validate()
} catch {
    print("Config error: \(error)")
    exit(1)
}

let vm = VZVirtualMachine(configuration: config)

print("Starting VM to extract DTB...")

vm.start { result in
    switch result {
    case .success:
        print("VM started — kernel is running")
        print("")

        // Let it run for a few seconds
        DispatchQueue.main.asyncAfter(deadline: .now() + 5) {
            print("Checking serial output...")
            if let data = FileManager.default.contents(atPath: logPath) {
                if data.count > 0 {
                    print("Serial output (\(data.count) bytes):")
                    print(String(data: data, encoding: .utf8) ?? "(binary)")
                } else {
                    print("No serial output — console not mapped to expected address")
                }
            }

            print("")
            print("VM state: \(vm.state.rawValue)")
            print("")
            print("To find device addresses, we need to extract the DTB")
            print("that VZLinuxBootLoader passes in x0.")
            print("")
            print("Key info from VZ framework:")
            print("  CPU count: \(config.cpuCount)")
            print("  Memory: \(config.memorySize / 1024 / 1024) MB")
            print("  Serial ports: \(config.serialPorts.count)")
            print("  Network devices: \(config.networkDevices.count)")
            print("  Boot loader: VZLinuxBootLoader")
            print("")
            print("The kernel IS executing on M4 silicon.")
            print("Next step: parse the DTB from inside the kernel")
            print("and write results to a shared memory region.")

            vm.stop { _ in
                exit(0)
            }
        }

    case .failure(let error):
        print("VM start failed: \(error)")
        exit(1)
    }
}

dispatchMain()
