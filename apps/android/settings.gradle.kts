// next.md's Phase 8/9 target: the Android consumer of the JNI bridges
// that `siar-transport-wifi-direct`, `siar-transport-wifi-aware`,
// `siar-transport-ble-android`, and `siar-transport-bluetooth-classic`
// were built for — every one of those crates' own doc comments named
// "apps/android doesn't exist yet" as the reason their JNI surface has
// no real caller. This module is that caller, finally.
//
// iOS is deliberately out of scope for this module — see this repo's
// root-level notes (and the memory of the conversation that produced
// this tree) for why: Android was the explicit target for this pass,
// not "mobile" generically. Nothing here assumes iOS will reuse this
// Gradle project; a real iOS app would be its own `apps/ios` Xcode
// project with its own bridge layer to the same Rust crates.

pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "siar-android"
include(":app")
