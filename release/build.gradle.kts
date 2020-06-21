import java.io.ByteArrayOutputStream
import java.io.IOException

val rustVersion: String by project

val releaseTasks = mutableMapOf<String, MutableList<TaskProvider<Task>>>()
val cleanTasks = mutableMapOf<String, MutableList<TaskProvider<Task>>>()

subprojects {
    // make sure we're actually a fully qualified project
    val parentProj = parent ?: return@subprojects
    parentProj.parent?.parent ?: return@subprojects

    val versionName = name
    val systemName = parentProj.name

    val dockerFile = parentProj.file("${parentProj.name}.Dockerfile")
    if (!dockerFile.exists()) {
        throw Exception("Dockerfile doesn't exist for $parentProj: $dockerFile")
    }

    val imageName = "paperd/$systemName:$versionName"
    val dockerImageFile = file("$buildDir/image.tar")
    val dockerImage = DockerImage(systemName, versionName, imageName, dockerFile, dockerImageFile)

    val dockerBuildTask = createDockerBuildTask(dockerImage, rustVersion)
    val runBuildTask = createRunBuildTask(dockerImage, true)
    runBuildTask {
        dependsOn(dockerBuildTask)
    }
    val runBuildNoConsoleTask = createRunBuildTask(dockerImage, false)
    runBuildNoConsoleTask {
        dependsOn(dockerBuildTask)
    }

    releaseTasks.computeIfAbsent(systemName) { mutableListOf() }.addAll(listOf(runBuildTask, runBuildNoConsoleTask))
    cleanTasks.computeIfAbsent(systemName) { mutableListOf() } += tasks.register("clean") {
        group = "clean"
        description = "Clean outputs of for ${systemName.capitalize()} $versionName"
        doLast {
            delete(file("build"))
        }
    }
}

val systemReleaseTasks = mutableListOf<TaskProvider<Task>>()
val systemCleanTasks = mutableListOf<TaskProvider<Task>>()
for ((systemName, buildTasks) in releaseTasks) {
    systemCleanTasks += findProject(":targets:$systemName")!!.tasks.register("buildReleases") {
        dependsOn(buildTasks)
        group = "paperd"
        description = "Build all releases for ${systemName.capitalize()}"
    }
}
for ((systemName, t) in cleanTasks) {
    systemCleanTasks += findProject(":targets:$systemName")!!.tasks.register("clean") {
        dependsOn(t)
        group = "clean"
        description = "Clean all outputs for ${systemName.capitalize()}"
    }
}

val buildReleases by findProject(":targets")!!.tasks.registering {
    dependsOn(systemReleaseTasks)
    group = "paperd"
    description = "Build all releases for all platforms"
}
tasks.register("buildReleases") {
    dependsOn(buildReleases)
    group = "paperd"
    description = "Alias of :targets:buildReleases"
}

val clean = findProject(":targets")!!.tasks.register("clean") {
    dependsOn(systemCleanTasks)
    group = "clean"
    description = "Clean all targets"
}
tasks.register("clean") {
    dependsOn(clean)
    group = "clean"
    description = "Alias for :targets:clean"
}

fun Project.createDockerBuildTask(
    dockerImage: DockerImage,
    rustVersion: String
): TaskProvider<Task> {
    return tasks.register("buildDockerImage") {
        group = "paperd"
        description = "Build Docker image for ${dockerImage.systemName.capitalize()} ${dockerImage.versionName}"

        inputs.file(dockerImage.dockerFile)

        outputs.file(dockerImage.dockerImageFile)

        doLast {
            docker(
                "build", "-t", dockerImage.imageName,
                "-f", dockerImage.dockerFile.absolutePath,
                "--build-arg", "version=${dockerImage.versionName}",
                "--build-arg", "rustVersion=$rustVersion",
                "."
            )

            // save the docker image to a file so it can be cached by gradle
            docker("save", "--output", dockerImage.dockerImageFile.absolutePath, dockerImage.imageName)
        }
    }
}

fun Project.createRunBuildTask(
    dockerImage: DockerImage,
    includeConsole: Boolean
): TaskProvider<Task> {
    val taskName = if (includeConsole) "buildRelease" else "buildReleaseNoConsole"
    return tasks.register(taskName) {
        group = "paperd"
        val extra = if (includeConsole) "" else " (no console)"
        description = "Build release for ${dockerImage.systemName.capitalize()} ${dockerImage.versionName}$extra"

        val baseDir = file("${rootProject.projectDir}/..").absoluteFile
        val inputSource = fileTree(baseDir) {
            include(
                "src/**/*.rs",
                "build.rs",
                "paperd-jni/src/**/*.rs",
                "paperd-lib/src/**/*.rs",
                "Cargo.*",
                "paperd-jni/Cargo.*",
                "paperd-lib/Cargo.*"
            )
        }
        inputs.files(inputSource)
        inputs.file(dockerImage.dockerImageFile)

        val (outputDir, targetFile) = if (includeConsole) {
            file("$buildDir/cargo-target") to
                file("${rootProject.buildDir}/paperd-${dockerImage.systemName}-${dockerImage.versionName}.tar.xz")
        } else {
            file("$buildDir/cargo-target-no-console") to
                file("${rootProject.buildDir}/" +
                    "paperd-${dockerImage.systemName}-${dockerImage.versionName}-no-console.tar.xz")
        }
        val outputFile = file("$outputDir/paperd.tar.xz")

        val registryDir = if (includeConsole) {
            file("$buildDir/cargo-registry")
        } else {
            file("$buildDir/cargo-registry-no-console")
        }
        outputs.file(targetFile)

        doLast {
            if (!outputDir.exists() && !outputDir.mkdirs()) {
                throw IOException("Failed to create output directory $outputDir")
            }
            if (!registryDir.exists() && !registryDir.mkdirs()) {
                throw IOException("Failed to create registry directory $registryDir")
            }

            docker("load", "--input", dockerImage.dockerImageFile.absolutePath)

            val uid = runCmd("id", "-u")
            val gid = runCmd("id", "-g")

            docker(
                "run", "--rm",
                "--user", "$uid:$gid",
                "-v", "$baseDir:/usr/src/paperd",
                "-v", "$outputDir:/usr/src/target",
                "-v", "$registryDir:/usr/local/cargo/registry",
                "-e", "INCLUDE_CONSOLE_BUILD=$includeConsole",
                dockerImage.imageName
            )

            outputFile.renameTo(targetFile)
        }
    }
}

fun docker(vararg args: String) {
    exec {
        commandLine("docker", *args)
    }
}

fun runCmd(vararg args: String): String {
    val os = ByteArrayOutputStream()
    exec {
        workingDir = projectDir
        commandLine(*args)
        standardOutput = os
    }
    return String(os.toByteArray()).trim()
}

data class DockerImage(
    val systemName: String,
    val versionName: String,
    val imageName: String,
    val dockerFile: File,
    val dockerImageFile: File
)
