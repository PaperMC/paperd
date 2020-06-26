paperd release build config
===========================

To ensure libraries properly link with `paperd` we build a binary for each platform. The base distros are listed in the
`targets` directory, and the `versions.txt` file in each distro directory lists the versions of each distro that we
provide builds for.

The way this Gradle project is set up, 2 Gradle sub-projects are created for each line in each `versions.txt` file. This
allows for parallel building of each platform. Each version has a `full` build project and a `no console` build project.

We use Docker to specify the build environment for each platform, so it must be installed on your machine in order to
run a release build. Your user on your machine must also have permission to submit Docker commands as well.

Running a build
---------------

### Warning (1):
**This build is set up to run parallel builds by default. Each of these builds include creating new Docker containers
and running a full release Cargo build. This can be _extremely_ taxing on your computer depending on how powerful your
machine is and how many threads it can run at the same time. In order to control how many threads Gradle uses for the
build you should specify the `org.gradle.workers.max` property for Gradle:
`./gradlew -Dorg.gradle.workers.max=<number> <task>`. You can also specify smaller builds rather than building
everything, as described below.**

### Warning (2):
**This build process creates a Docker image for each version listed in the `versions.txt` file for each distro listed in
the `targets` directory. It then performs 2 separate Cargo builds per Docker image and stores the Cargo registry and
build outputs in the `build/` directory separately for each build configuration. This will download and generate a
significant amount of data, on the order of _30GB or more_. Limit the number of platforms you build if you want to reduce
the amount of data created.**

#### Build all targets for all platforms:
```sh
./gradlew buildReleases 
```

#### Build all targets for a single platform:
```sh
./gradlew :targets:<targetName>:buildReleases
Example:
./gradlew :targets:debian:buildReleases
```

#### Build the console and no console target for a single platform version
```sh
./gradlew :targets:<targetName>:<versionName>:buildReleases
Example:
./gradlew :targets:debian:buster:buildReleases
```

#### Build a single target
```sh
./gradlew :targets:<targetName>:<versionName>:<full|noConsole>:buildRelease
Example:
./gradlew :targets:debian:buster:full:buildRelease
```

#### Clean build outputs
Clean tasks follow the exact same pattern as shown above. The only difference is there is a single `clean` task
for both `buildReleaseFull` and `buildReleaseNoConsole` tasks for each version of each platform.
```sh
./gradlew clean # clean all outputs for all versions of all platforms
./gradlew :targets:<targetName>:clean # clean all builds for all versions of a platform
./gradlew :targets:<targetName>:<versionName>:clean # clean all outputs for a version of a platform
./gradlew :targets:<targetName>:<versionName>:full:clean # clean outputs for a single target
```
