// Original painted signs. Run from the repository root with swift rust/tools/make_postcard_signs.swift.
import AppKit

let size = NSSize(width: 1024, height: 256)
func colour(_ rgb: UInt32) -> NSColor {
    NSColor(srgbRed: Double((rgb >> 16) & 255) / 255, green: Double((rgb >> 8) & 255) / 255,
            blue: Double(rgb & 255) / 255, alpha: 1)
}
func sign(_ file: String, _ title: String, _ caption: String, _ paint: UInt32, _ ink: UInt32, _ accent: UInt32) throws {
    let image = NSImage(size: size, flipped: false) { bounds in
        colour(paint).setFill()
        bounds.fill()
        let border = NSBezierPath(roundedRect: bounds.insetBy(dx: 26, dy: 26), xRadius: 17, yRadius: 17)
        colour(accent).setStroke()
        border.lineWidth = 6
        border.stroke()
        // Broad, low-contrast wear shows the paint surface without making the lettering noisy.
        for index in 0..<60 {
            let x = CGFloat(32 + (index * 307 % 950))
            let y = CGFloat(28 + (index * 143 % 202))
            colour(ink).withAlphaComponent(0.055).setFill()
            NSRect(x: x, y: y, width: CGFloat(5 + index * 9 % 28), height: 2).fill()
        }
        for x in [43.0, 981.0] {
            for y in [43.0, 213.0] {
                colour(ink).withAlphaComponent(0.48).setFill()
                NSBezierPath(ovalIn: NSRect(x: x - 4, y: y - 4, width: 8, height: 8)).fill()
            }
        }
        func line(_ text: String, y: CGFloat, font: NSFont, rgb: UInt32, tracking: CGFloat) {
            let string = NSAttributedString(string: text, attributes: [
                .font: font, .foregroundColor: colour(rgb), .kern: tracking,
            ])
            string.draw(at: NSPoint(x: (size.width - string.size().width) / 2, y: y))
        }
        line(title, y: 87, font: .systemFont(ofSize: 96, weight: .black), rgb: ink, tracking: 0)
        line(caption, y: 47, font: .monospacedSystemFont(ofSize: 28, weight: .bold), rgb: accent, tracking: 2)
        return true
    }
    let bitmap = NSBitmapImageRep(data: image.tiffRepresentation!)!
    let url = URL(fileURLWithPath: "rust/crates/bbq_app/assets/textures/" + file + ".png")
    try bitmap.representation(using: .png, properties: [:])!.write(to: url)
}

try sign("postcard_bbq", "DAZZA’S BARBIE", "SATURDAYS  •  SNAGS FROM NOON", 0xe4c59b, 0x453925, 0x914f36)
try sign("postcard_servo", "BUSH SERVO", "SMOKO STOP  •  EST. ’76", 0xe2bc87, 0x553e2c, 0x8b4e35)
try sign("postcard_heist", "TEDDY DEPOT", "CENTRAL SCOREBOARD", 0x704437, 0xffe4ac, 0xffba70)
