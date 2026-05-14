// Sphragis — Native M4 VM Launcher
// Uses Apple's Virtualization.framework to run Sphragis
// directly on M4 silicon. No emulation — your actual CPU
// cores execute the Sphragis kernel at native speed.
//
// Usage: swift Sphragis_VM.swift <path_to_kernel>

import Virtualization
import Foundation

let kernelPath: String
if CommandLine.arguments.count > 1 {
    kernelPath = CommandLine.arguments[1]
} else {
    // Default path
    let home = FileManager.default.homeDirectoryForCurrentUser.path
    kernelPath = "\(home)/Sphragis/target/aarch64-unknown-none/release/sphragis"
}

guard FileManager.default.fileExists(atPath: kernelPath) else {
    print("Error: Kernel not found at \(kernelPath)")
    print("Build it first: cd ~/Sphragis && cargo build --release")
    exit(1)
}

print("""
══════════════════════════════════════════════
  SPHRAGIS — Native M4 Virtual Machine
══════════════════════════════════════════════
  Kernel: \(kernelPath)
  CPU:    Apple M4 (native execution)
  RAM:    256 MB
  Mode:   Virtualization.framework (hardware VM)
══════════════════════════════════════════════
""")

// ─── VM Configuration ───

let config = VZVirtualMachineConfiguration()

// Platform: generic ARM64
let platform = VZGenericPlatformConfiguration()
config.platform = platform

// CPU: 2 cores
config.cpuCount = 2

// RAM: 256 MB
config.memorySize = 256 * 1024 * 1024

// Boot loader: ARM64 Linux Image binary
let bootLoader = VZLinuxBootLoader(kernelURL: URL(fileURLWithPath: kernelPath))
bootLoader.commandLine = "console=hvc0"
config.bootLoader = bootLoader

// Serial console — output to file + stdout
let serialPort = VZVirtioConsoleDeviceSerialPortConfiguration()
let logPath = NSTemporaryDirectory() + "sphragis_serial.log"
FileManager.default.createFile(atPath: logPath, contents: nil)
let logHandle = FileHandle(forWritingAtPath: logPath)!
let stdioAttachment = VZFileHandleSerialPortAttachment(
    fileHandleForReading: FileHandle.standardInput,
    fileHandleForWriting: logHandle
)
serialPort.attachment = stdioAttachment
config.serialPorts = [serialPort]
print("  Serial log: \(logPath)")

// Network — NAT networking (like QEMU user mode)
let networkDevice = VZVirtioNetworkDeviceConfiguration()
networkDevice.attachment = VZNATNetworkDeviceAttachment()
config.networkDevices = [networkDevice]

// Storage — 1GB virtual disk for BatFS
let diskURL = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent("sphragis_disk.img")
if !FileManager.default.fileExists(atPath: diskURL.path) {
    // Create a 1GB disk image
    FileManager.default.createFile(atPath: diskURL.path, contents: nil)
    let handle = try! FileHandle(forWritingTo: diskURL)
    handle.truncateFile(atOffset: 1024 * 1024 * 1024) // 1GB
    handle.closeFile()
    print("  Created 1GB virtual disk")
}
let diskAttachment = try! VZDiskImageStorageDeviceAttachment(url: diskURL, readOnly: false)
let diskDevice = VZVirtioBlockDeviceConfiguration(attachment: diskAttachment)
config.storageDevices = [diskDevice]

// Entropy — random number source
config.entropyDevices = [VZVirtioEntropyDeviceConfiguration()]

// Memory balloon
config.memoryBalloonDevices = [VZVirtioTraditionalMemoryBalloonDeviceConfiguration()]

// ─── Validate Configuration ───

do {
    try config.validate()
    print("  VM configuration valid ✓")
} catch {
    print("  VM configuration error: \(error)")
    exit(1)
}

// ─── Create and Start VM ───

let vm = VZVirtualMachine(configuration: config)

print("  Starting Sphragis...")
print("══════════════════════════════════════════════")
print("")

// Set up terminal for raw mode (pass keyboard directly to VM)
var originalTermios = termios()
tcgetattr(STDIN_FILENO, &originalTermios)
var rawTermios = originalTermios
cfmakeraw(&rawTermios)
tcsetattr(STDIN_FILENO, TCSANOW, &rawTermios)

// Restore terminal on exit
func cleanup() {
    tcsetattr(STDIN_FILENO, TCSANOW, &originalTermios)
    print("\n\nBat_OS VM terminated.")
}
atexit { cleanup() }
signal(SIGINT) { _ in
    cleanup()
    exit(0)
}

// Start the VM
let semaphore = DispatchSemaphore(value: 0)

vm.start { result in
    switch result {
    case .success:
        break // VM is running
    case .failure(let error):
        tcsetattr(STDIN_FILENO, TCSANOW, &originalTermios)
        print("Failed to start VM: \(error)")
        exit(1)
    }
}

// Wait forever (Ctrl+C to exit)
dispatchMain()
