import java.io.File
import org.apache.tools.ant.taskdefs.condition.Os
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.logging.LogLevel
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction

open class BuildTask : DefaultTask() {
    @Input
    var rootDirRel: String? = null
    @Input
    var target: String? = null
    @Input
    var release: Boolean? = null
    // looks like tauri itself cannot find pnpm. oh my god.
    private fun findPnpm(): String {
        val candidates = listOf(
            "/opt/homebrew/bin/pnpm",
            "/usr/local/bin/pnpm",
            System.getenv("HOME") + "/.nvm/versions/node/*/bin/pnpm",
            "pnpm"
        )
        for (candidate in candidates) {
            if (candidate.contains("*")) {
                val expanded = project.fileTree(mapOf("dir" to candidate.substringBeforeLast("/"), "include" to candidate.substringAfterLast("/")))
                if (!expanded.isEmpty) {
                    return expanded.singleFile.absolutePath
                }
            } else {
                val file = File(candidate)
                if (file.exists() && file.canExecute()) {
                    return file.absolutePath
                }
            }
        }
        return "pnpm"
    }

    @TaskAction
    fun assemble() {
        val executable = findPnpm()
        try {
            runTauriCli(executable)
        } catch (e: Exception) {
            if (Os.isFamily(Os.FAMILY_WINDOWS)) {
                // Try different Windows-specific extensions
                val fallbacks = listOf(
                    "$executable.exe",
                    "$executable.cmd",
                    "$executable.bat",
                )
                
                var lastException: Exception = e
                for (fallback in fallbacks) {
                    try {
                        runTauriCli(fallback)
                        return
                    } catch (fallbackException: Exception) {
                        lastException = fallbackException
                    }
                }
                throw lastException
            } else {
                throw e;
            }
        }
    }

    fun runTauriCli(executable: String) {
        val rootDirRel = rootDirRel ?: throw GradleException("rootDirRel cannot be null")
        val target = target ?: throw GradleException("target cannot be null")
        val release = release ?: throw GradleException("release cannot be null")
        val args = listOf("tauri", "android", "android-studio-script");

        project.exec {
            workingDir(File(project.projectDir, rootDirRel))
            executable(executable)
            args(args)
            if (project.logger.isEnabled(LogLevel.DEBUG)) {
                args("-vv")
            } else if (project.logger.isEnabled(LogLevel.INFO)) {
                args("-v")
            }
            if (release) {
                args("--release")
            }
            args(listOf("--target", target))
        }.assertNormalExitValue()
    }
}