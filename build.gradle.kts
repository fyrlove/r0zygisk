import com.android.build.gradle.LibraryExtension
import java.io.ByteArrayOutputStream

plugins {
    alias(libs.plugins.agp.lib) apply false
}

fun String.execute(currentWorkingDir: File = file("/")): String {
    val byteOut = ByteArrayOutputStream()
    project.exec {
        workingDir = currentWorkingDir
        commandLine = split("\\s".toRegex())
        standardOutput = byteOut
    }
    return String(byteOut.toByteArray()).trim()
}

val gitDir = rootDir.resolve(".git")
val gitCommitCount = if (gitDir.exists()) {
    "/usr/bin/git -C ${rootDir.absolutePath} rev-list HEAD --count".execute().toInt()
} else {
    1
}
val gitCommitHash = if (gitDir.exists()) {
    "/usr/bin/git -C ${rootDir.absolutePath} rev-parse --verify --short HEAD".execute()
} else {
    "unknown"
}

val moduleId by extra("r0z")
val moduleName by extra("r0z")
val verName by extra("v1.0.3")
val verCode by extra(4)
val commitHash by extra(gitCommitHash)
val minKsuVersion by extra(10940)
val minKsudVersion by extra(11425)
val maxKsuVersion by extra(20000)
val minMagiskVersion by extra(26402)

val androidMinSdkVersion by extra(26)
val androidTargetSdkVersion by extra(34)
val androidCompileSdkVersion by extra(34)
val androidBuildToolsVersion by extra("34.0.0")
val androidCompileNdkVersion by extra("26.1.10909125")
val androidSourceCompatibility by extra(JavaVersion.VERSION_11)
val androidTargetCompatibility by extra(JavaVersion.VERSION_11)

tasks.register("Delete", Delete::class) {
    delete(rootProject.buildDir)
}

fun Project.configureBaseExtension() {
    extensions.findByType(LibraryExtension::class)?.run {
        namespace = "com.r0ysue.r0z"
        compileSdk = androidCompileSdkVersion
        ndkVersion = androidCompileNdkVersion
        buildToolsVersion = androidBuildToolsVersion

        defaultConfig {
            minSdk = androidMinSdkVersion
            targetSdk = androidTargetSdkVersion
        }

        lint {
            abortOnError = true
        }
    }
}

subprojects {
    plugins.withId("com.android.library") {
        configureBaseExtension()
    }
}
