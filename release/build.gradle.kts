import java.io.ByteArrayOutputStream
import java.io.IOException
import org.gradle.api.tasks.Copy

val rustVersion: String by project

val releaseTasks = mutableMapOf<String, MutableMap<String, MutableList<TaskProvider<Task>>>>()
val cleanTasks = mutableMapOf<String, MutableMap<String, MutableList<TaskProvider<Task>>>>()

subprojects {
    // make sure we're actually a fully qualified project
    val parentProj = parent ?: return@subprojects
    val grandParentProj = parentProj.parent ?: return@subprojects
    parentProj.parent?.parent?.parent ?: return@subprojects

    val typeName = name
    val versionName = parentProj.name
    val systemName = grandParentProj.name

    val dockerFile = grandParentProj.file("${grandParentProj.name}.Dockerfile")
    if (!dockerFile.exists()) {
        throw Exception("Dockerfile doesn't exist for $grandParentProj: $dockerFile")
    }

    val imageName = "paperd/$systemName:$versionName"
    val dockerImage = DockerImage(systemName, versionName, imageName, dockerFile)

    // Make sure the docker build task is set up on the parent project
    val dockerBuildTask: TaskProvider<Task> = try {
        val copyDockerIgnore = parentProj.copyDockerIgnore()
        val dockerBuildTask = parentProj.createDockerBuildTask(dockerImage, rustVersion)
        dockerBuildTask {
            dependsOn(copyDockerIgnore)
        }
        dockerBuildTask
    } catch (e: Exception) {
        parentProj.tasks.named("buildDockerImage")
    }

    val copyDockerIgnore = copyDockerIgnore()
    val runBuildTask = createRunBuildTask(dockerImage, typeName == "full")
    runBuildTask {
        dependsOn(dockerBuildTask, copyDockerIgnore)
    }

    val clean by tasks.registering {
        group = "clean"
        val extra = if (typeName == "full") "" else " (no console)"
        description = "Clean outputs of for ${systemName.capitalize()} $versionName$extra"
        doLast {
            delete(file("build"))
            delete(file(".dockerignore"))
            delete(runBuildTask)
        }
    }

    //
    // Build task hierarchy
    val versionTargetBuild = parentProj.findOrCreateTask("buildReleases") {
        group = "paperd"
        description = "Build all targets for ${systemName.capitalize()} $versionName"
    }.apply {
        configure {
            dependsOn(runBuildTask)
        }
    }
    val systemTargetBuild = grandParentProj.findOrCreateTask("buildReleases") {
        group = "paperd"
        description = "Build all targets for all versions for ${systemName.capitalize()}"
    }.apply {
        configure {
            dependsOn(versionTargetBuild)
        }
    }
    grandParentProj.parent!!.findOrCreateTask("buildReleases") {
        group = "paperd"
        description = "Build all targets for all versions of all platforms"
    }.apply {
        configure {
            dependsOn(systemTargetBuild)
        }
    }

    //
    // Clean task hierarchy
    val versionTargetClean = parentProj.findOrCreateTask("clean") {
        group = "clean"
        description = "Clean all outputs for ${systemName.capitalize()} $versionName"
        doLast {
            delete(parentProj.file(".dockerignore"))
        }
    }.apply {
        configure {
            dependsOn(clean)
        }
    }
    val systemTargetClean = grandParentProj.findOrCreateTask("clean") {
        group = "clean"
        description = "Clean all outputs for all versions ${systemName.capitalize()}"
    }.apply {
        configure {
            dependsOn(versionTargetClean)
        }
    }
    grandParentProj.parent!!.findOrCreateTask("clean") {
        group = "clean"
        description = "Clean all outputs for all versions of all platforms"
    }.apply {
        configure {
            dependsOn(systemTargetClean)
        }
    }
}

tasks.register("buildReleases") {
    dependsOn(findProject(":targets")!!.tasks.named("buildReleases"))
    group = "paperd"
    description = "Alias of :targets:buildReleases"
}
tasks.register("clean") {
    dependsOn(findProject(":targets")!!.tasks.named("clean"))
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

        doLast {
            docker(
                "build", "-t", dockerImage.imageName,
                "-f", dockerImage.dockerFile.absolutePath,
                "--build-arg", "version=${dockerImage.versionName}",
                "--build-arg", "rustVersion=$rustVersion",
                "."
            )
        }
    }
}

fun Project.createRunBuildTask(
    dockerImage: DockerImage,
    includeConsole: Boolean
): TaskProvider<Task> {
    return tasks.register("buildRelease") {
        group = "paperd"
        val extra = if (includeConsole) "" else " (no console)"
        description = "Build release for ${dockerImage.systemName.capitalize()} ${dockerImage.versionName}$extra"

        val baseDir = rootProject.file("..").absoluteFile
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

        val targetFileName = if (includeConsole) {
            "paperd-${dockerImage.systemName}-${dockerImage.versionName}.tar.xz"
        } else {
            "paperd-${dockerImage.systemName}-${dockerImage.versionName}-no-console.tar.xz"
        }
        val targetFile = file("${rootProject.buildDir}/$targetFileName")
        outputs.file(targetFile)

        val outputDir = file("$buildDir/cargo-target")
        val outputFile = file("$outputDir/paperd.tar.xz")
        val registryDir = file("$buildDir/cargo-registry")

        doLast {
            if (!outputDir.exists() && !outputDir.mkdirs()) {
                throw IOException("Failed to create output directory $outputDir")
            }
            if (!registryDir.exists() && !registryDir.mkdirs()) {
                throw IOException("Failed to create registry directory $registryDir")
            }

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

fun Project.copyDockerIgnore(): TaskProvider<out Task> {
    return tasks.register("copyDockerIgnore", Copy::class) {
        from("${rootProject.projectDir}/targets/.dockerignore")
        into(projectDir)
    }
}

fun Project.docker(vararg args: String) {
    exec {
        commandLine("docker", *args)
        workingDir = projectDir

    }
}

inline fun Project.findOrCreateTask(name: String, crossinline config: (Task).() -> Unit): TaskProvider<Task> {
    return try {
        tasks.register(name) {
            config()
        }
    } catch (e: Exception) {
        tasks.named(name)
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
    val dockerFile: File
)
