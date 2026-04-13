// Bat_OS — Native M4 VM with GUI Display
// Uses VZVirtioGraphicsDeviceConfiguration for visual output.
// The kernel writes to a framebuffer that macOS displays in a window.

import Virtualization
import AppKit
import Foundation

let home = FileManager.default.homeDirectoryForCurrentUser.path
let kernelPath: String
if CommandLine.arguments.count > 1 {
    kernelPath = CommandLine.arguments[1]
} else {
    kernelPath = "\(home)/Bat_OS/target/bat_os_image.bin"
}

guard FileManager.default.fileExists(atPath: kernelPath) else {
    print("Kernel not found: \(kernelPath)")
    exit(1)
}

// Create the app
let app = NSApplication.shared
app.setActivationPolicy(.regular)

// VM Configuration
let config = VZVirtualMachineConfiguration()
let platform = VZGenericPlatformConfiguration()
config.platform = platform
config.cpuCount = 2
config.memorySize = 512 * 1024 * 1024  // 512MB

let bootLoader = VZLinuxBootLoader(kernelURL: URL(fileURLWithPath: kernelPath))
bootLoader.commandLine = "console=hvc0"
config.bootLoader = bootLoader

// Serial (write to file for debugging)
let serialPort = VZVirtioConsoleDeviceSerialPortConfiguration()
let logPath = NSTemporaryDirectory() + "bat_os_gui.log"
FileManager.default.createFile(atPath: logPath, contents: nil)
let logHandle = FileHandle(forWritingAtPath: logPath)!
serialPort.attachment = VZFileHandleSerialPortAttachment(
    fileHandleForReading: FileHandle(forReadingAtPath: "/dev/null")!,
    fileHandleForWriting: logHandle
)
config.serialPorts = [serialPort]

// GPU Display — this gives the VM a framebuffer!
let graphicsDevice = VZVirtioGraphicsDeviceConfiguration()
graphicsDevice.scanouts = [
    VZVirtioGraphicsScanoutConfiguration(widthInPixels: 1280, heightInPixels: 800)
]
config.graphicsDevices = [graphicsDevice]

// Keyboard
config.keyboards = [VZUSBKeyboardConfiguration()]

// Mouse
config.pointingDevices = [VZUSBScreenCoordinatePointingDeviceConfiguration()]

// Network
let netDev = VZVirtioNetworkDeviceConfiguration()
netDev.attachment = VZNATNetworkDeviceAttachment()
config.networkDevices = [netDev]

// Storage
let diskURL = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent("bat_os_disk.img")
if !FileManager.default.fileExists(atPath: diskURL.path) {
    FileManager.default.createFile(atPath: diskURL.path, contents: nil)
    let handle = try! FileHandle(forWritingTo: diskURL)
    handle.truncateFile(atOffset: 1024 * 1024 * 1024)
    handle.closeFile()
}
let diskAttachment = try! VZDiskImageStorageDeviceAttachment(url: diskURL, readOnly: false)
config.storageDevices = [VZVirtioBlockDeviceConfiguration(attachment: diskAttachment)]

config.entropyDevices = [VZVirtioEntropyDeviceConfiguration()]
config.memoryBalloonDevices = [VZVirtioTraditionalMemoryBalloonDeviceConfiguration()]

do {
    try config.validate()
    print("VM configuration valid")
} catch {
    print("Config error: \(error)")
    exit(1)
}

let vm = VZVirtualMachine(configuration: config)

// Create window with VM view
let window = NSWindow(
    contentRect: NSRect(x: 100, y: 100, width: 1280, height: 800),
    styleMask: [.titled, .closable, .miniaturizable, .resizable],
    backing: .buffered,
    defer: false
)
window.title = "Bat_OS — M4 Native VM"
window.center()

let vmView = VZVirtualMachineView()
vmView.virtualMachine = vm
vmView.capturesSystemKeys = true
window.contentView = vmView
window.makeKeyAndOrderFront(nil)

// Start VM
vm.start { result in
    switch result {
    case .success:
        print("Bat_OS running on M4 silicon!")
    case .failure(let error):
        print("VM failed: \(error)")
    }
}

app.activate(ignoringOtherApps: true)
app.run()
