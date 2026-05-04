// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "AfterLivie",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(name: "AfterLivie", targets: ["AfterLivie"])
    ],
    targets: [
        .executableTarget(
            name: "AfterLivie",
            path: "Sources/AfterLivie"
        )
    ]
)
