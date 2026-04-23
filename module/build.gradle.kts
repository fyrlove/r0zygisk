import android.databinding.tool.ext.capitalizeUS
import java.security.MessageDigest
import org.apache.tools.ant.filters.ReplaceTokens

import org.apache.tools.ant.filters.FixCrLfFilter

import org.apache.commons.codec.binary.Hex
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.security.KeyFactory
import java.security.KeyPairGenerator
import java.security.Signature
import java.security.interfaces.EdECPrivateKey
import java.security.interfaces.EdECPublicKey
import java.security.spec.EdECPrivateKeySpec
import java.security.spec.NamedParameterSpec
import java.util.TreeSet

plugins {
    alias(libs.plugins.agp.lib)
}

val moduleId: String by rootProject.extra
val moduleName: String by rootProject.extra
val verCode: Int by rootProject.extra
val verName: String by rootProject.extra
val minKsuVersion: Int by rootProject.extra
val minKsudVersion: Int by rootProject.extra
val maxKsuVersion: Int by rootProject.extra
val minMagiskVersion: Int by rootProject.extra
val commitHash: String by rootProject.extra

android.buildFeatures {
    androidResources = false
    buildConfig = false
}

androidComponents.onVariants { variant ->
    val variantLowered = variant.name.lowercase()
    val variantCapped = variant.name.capitalizeUS()
    val buildTypeLowered = variant.buildType?.lowercase()

    val moduleDir = layout.buildDirectory.dir("outputs/module/$variantLowered")
    val zipFileName = "$moduleName-$verName-$verCode-$buildTypeLowered.zip".replace(' ', '-')

    val prepareModuleFilesTask = task<Sync>("prepareModuleFiles$variantCapped") {
        group = "module"
        dependsOn(
            ":loader:assemble$variantCapped",
            ":r0zd:buildAndStrip",
        )
        into(moduleDir)
        from("${rootProject.projectDir}/README.md")
        from("$projectDir/src") {
            exclude("module.prop", "customize.sh", "post-fs-data.sh", "service.sh", "r0z-ctl.sh", "mazoku")
            filter<FixCrLfFilter>("eol" to FixCrLfFilter.CrLf.newInstance("lf"))
        }
        from("$projectDir/src") {
            include("module.prop")
            expand(
                "moduleId" to moduleId,
                "moduleName" to moduleName,
                "versionName" to "$verName ($verCode-$variantLowered)",
                "versionCode" to verCode
            )
        }
        from("$projectDir/src/mazoku")
        from("$projectDir/src") {
            include("customize.sh", "post-fs-data.sh", "service.sh", "r0z-ctl.sh")
            val tokens = mapOf(
                "DEBUG" to if (buildTypeLowered == "debug") "true" else "false",
                "MIN_KSU_VERSION" to "$minKsuVersion",
                "MIN_KSUD_VERSION" to "$minKsudVersion",
                "MAX_KSU_VERSION" to "$maxKsuVersion",
                "MIN_MAGISK_VERSION" to "$minMagiskVersion",
            )
            filter<ReplaceTokens>("tokens" to tokens)
            filter<FixCrLfFilter>("eol" to FixCrLfFilter.CrLf.newInstance("lf"))
        }
        into("bin") {
            from(project(":r0zd").layout.buildDirectory.file("rustJniLibs/android"))
            include("**/r0zd")
        }
        into("lib") {
            from(project(":loader").layout.buildDirectory.file("intermediates/stripped_native_libs/$variantLowered/strip${variantCapped}DebugSymbols/out/lib"))
        }

        val root = moduleDir.get()

        doLast {
            if (file("private_key").exists()) {
                println("=== Guards the peace of Machikado ===")
                val privateKey = file("private_key").readBytes()
                val publicKey = file("public_key").readBytes()
                val namedSpec = NamedParameterSpec("ed25519")
                val privKeySpec = EdECPrivateKeySpec(namedSpec, privateKey)
                val kf = KeyFactory.getInstance("ed25519")
                val privKey = kf.generatePrivate(privKeySpec);
                val sig = Signature.getInstance("ed25519")
                fun File.sha(realFile: File? = null) {
                    val path = this.path.replace("\\", "/")
                    sig.update(this.name.toByteArray())
                    sig.update(0) // null-terminated string
                    val real = realFile ?: this
                    val buffer = ByteBuffer.allocate(8)
                        .order(ByteOrder.LITTLE_ENDIAN)
                        .putLong(real.length())
                        .array()
                    sig.update(buffer)
                    real.forEachBlock { bytes, size ->
                        sig.update(bytes, 0, size)
                    }
                }

                fun getSign(name: String, abi32: String, abi64: String) {
                    val set = TreeSet<Pair<File, File?>> { o1, o2 ->
                        o1.first.path.replace("\\", "/")
                            .compareTo(o2.first.path.replace("\\", "/"))
                    }
                    set.add(Pair(root.file("module.prop").asFile, null))
                    set.add(Pair(root.file("sepolicy.rule").asFile, null))
                    set.add(Pair(root.file("post-fs-data.sh").asFile, null))
                    set.add(Pair(root.file("service.sh").asFile, null))
                    set.add(Pair(root.file("mazoku").asFile, null))
                    set.add(
                        Pair(
                            root.file("system/lib/libr0zgk.so").asFile,
                            root.file("lib/$abi32/libr0zgk.so").asFile
                        )
                    )
                    set.add(
                        Pair(
                            root.file("system/lib64/libr0zgk.so").asFile,
                            root.file("lib/$abi64/libr0zgk.so").asFile
                        )
                    )
                    set.add(
                        Pair(
                            root.file("system/lib/libzn_loader.so").asFile,
                            root.file("lib/$abi32/libzn_loader.so").asFile
                        )
                    )
                    set.add(
                        Pair(
                            root.file("system/lib64/libzn_loader.so").asFile,
                            root.file("lib/$abi64/libzn_loader.so").asFile
                        )
                    )
                    set.add(
                        Pair(
                            root.file("system/lib/libpayload.so").asFile,
                            root.file("lib/$abi32/libpayload.so").asFile
                        )
                    )
                    set.add(
                        Pair(
                            root.file("system/lib64/libpayload.so").asFile,
                            root.file("lib/$abi64/libpayload.so").asFile
                        )
                    )
                    set.add(
                        Pair(
                            root.file("bin/r0zd32").asFile,
                            root.file("bin/$abi32/r0zd").asFile
                        )
                    )
                    set.add(
                        Pair(
                            root.file("bin/r0zd64").asFile,
                            root.file("bin/$abi64/r0zd").asFile
                        )
                    )
                    set.add(
                        Pair(
                            root.file("bin/r0z-ctl").asFile,
                            root.file("r0z-ctl.sh").asFile
                        )
                    )
                    sig.initSign(privKey)
                    set.forEach { it.first.sha(it.second) }
                    val signFile = root.file(name).asFile
                    signFile.writeBytes(sig.sign())
                    signFile.appendBytes(publicKey)
                }

                getSign("machikado.arm", "armeabi-v7a", "arm64-v8a")
                getSign("machikado.x86", "x86", "x86_64")
            } else {
                println("no private_key found, this build will not be signed")
                root.file("machikado.arm").asFile.createNewFile()
                root.file("machikado.x86").asFile.createNewFile()
            }

            fileTree(moduleDir).visit {
                if (isDirectory) return@visit
                val md = MessageDigest.getInstance("SHA-256")
                file.forEachBlock(4096) { bytes, size ->
                    md.update(bytes, 0, size)
                }
                file(file.path + ".sha256").writeText(Hex.encodeHexString(md.digest()))
            }
        }
    }

    val zipTask = task<Zip>("zip$variantCapped") {
        group = "module"
        dependsOn(prepareModuleFilesTask)
        archiveFileName.set(zipFileName)
        destinationDirectory.set(layout.buildDirectory.file("outputs/release").get().asFile)
        from(moduleDir)
    }

    val pushTask = task<Exec>("push$variantCapped") {
        group = "module"
        dependsOn(zipTask)
        commandLine("adb", "push", zipTask.outputs.files.singleFile.path, "/data/local/tmp")
    }

    val installKsuTask = task("installKsu$variantCapped") {
        group = "module"
        dependsOn(pushTask)
        doLast {
            exec {
                commandLine(
                    "adb", "shell", "echo",
                    "/data/adb/ksud module install /data/local/tmp/$zipFileName",
                    "> /data/local/tmp/install.sh"
                )
            }
            exec { commandLine("adb", "shell", "chmod", "755", "/data/local/tmp/install.sh") }
            exec { commandLine("adb", "shell", "su", "-c", "/data/local/tmp/install.sh") }
        }
    }

    val installMagiskTask = task<Exec>("installMagisk$variantCapped") {
        group = "module"
        dependsOn(pushTask)
        commandLine("adb", "shell", "su", "-M", "-c", "magisk --install-module /data/local/tmp/$zipFileName")
    }

    task<Exec>("installKsuAndReboot$variantCapped") {
        group = "module"
        dependsOn(installKsuTask)
        commandLine("adb", "reboot")
    }

    task<Exec>("installMagiskAndReboot$variantCapped") {
        group = "module"
        dependsOn(installMagiskTask)
        commandLine("adb", "reboot")
    }
}
