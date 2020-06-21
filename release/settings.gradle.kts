import java.io.IOException
import java.nio.file.Files
import java.nio.file.Path

rootProject.name = "paperd"

val targetsDir = file("targets")
val targetsPath: Path = targetsDir.toPath()

targetsDir.walkTopDown()
    .filter { it.isFile && it.name == "versions.txt" }
    .map { it.toPath() }
    .forEach { path ->
        val relative = targetsPath.relativize(path)
        val systemName = relative.getName(0)
        Files.lines(path)
            .filter { it.isNotBlank() }
            .forEach { line ->
                val projectName = "targets:$systemName:$line"
                val projectDir = "${rootProject.projectDir}/build/$systemName/$line"
                file(projectDir).let { dir ->
                    if (!dir.exists()) {
                        if (!dir.mkdirs()) {
                            throw IOException("Failed to create $dir")
                        }
                    }
                }
                include(projectName)
                project(":$projectName").projectDir = file(projectDir)
            }
    }
