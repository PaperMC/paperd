import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths

rootProject.name = "paperd"

val rootProjectDir: Path = rootProject.projectDir.toPath()
val targetsDir: Path = file("targets").toPath()

Files.walk(targetsDir)
    .filter { Files.isRegularFile(it) && it.fileName.toString() == "versions.txt" }
    .forEach { path ->
        val systemName = targetsDir.relativize(path).getName(0).toString()
        Files.lines(path)
            .filter { it.isNotBlank() }
            .forEach { line ->
                val systemProjectName = "targets:$systemName:$line"
                val fullProjectName = "$systemProjectName:full"
                val fullProjectDir = rootProjectDir.resolve(Paths.get("build", systemName, line, "full"))
                Files.createDirectories(fullProjectDir)
                val noConsoleProjectName = "$systemProjectName:noConsole"
                val noConsoleProjectDir = rootProjectDir.resolve(Paths.get("build", systemName, line, "noConsole"))
                Files.createDirectories(noConsoleProjectDir)

                include(fullProjectName)
                project(":$fullProjectName").projectDir = fullProjectDir.toFile()
                include(noConsoleProjectName)
                project(":$noConsoleProjectName").projectDir = noConsoleProjectDir.toFile()
                project(":$systemProjectName").projectDir = fullProjectDir.parent.toFile()
            }
    }
