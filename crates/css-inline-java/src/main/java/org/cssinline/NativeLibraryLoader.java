package org.cssinline;

import java.io.*;
import java.nio.file.*;

class NativeLibraryLoader {
	private static final String LIBRARY_NAME = "css_inline";
	private static boolean loaded = false;

	static synchronized void loadLibrary() {
		if (loaded)
			return;

		try {
			// Try system library first (for development)
			System.loadLibrary(LIBRARY_NAME);
			loaded = true;
			return;
		} catch (UnsatisfiedLinkError e) {
			// Fall back to bundled library
		}

		String platform = detectPlatform();
		String libraryPath = "/org/cssinline/native/" + platform + "/" + getLibraryFileName();

		try (InputStream is = NativeLibraryLoader.class.getResourceAsStream(libraryPath)) {
			if (is == null) {
				throw new UnsatisfiedLinkError("Native library not found: " + libraryPath
						+ ". Available platforms: linux-x86_64, darwin-x86_64, darwin-aarch64, win32-x86_64");
			}

			// Extract to temporary file
			Path tempDir = Files.createTempDirectory("css-inline-native");
			Path tempFile = tempDir.resolve(getLibraryFileName());
			Files.copy(is, tempFile, StandardCopyOption.REPLACE_EXISTING);

			// Load library
			System.load(tempFile.toAbsolutePath().toString());
			loaded = true;

			Runtime.getRuntime().addShutdownHook(new Thread(() -> {
				try {
					Files.deleteIfExists(tempFile);
					Files.deleteIfExists(tempDir);
				} catch (IOException ignored) {
				}
			}));

		} catch (IOException e) {
			throw new UnsatisfiedLinkError("Failed to load native library: " + e.getMessage());
		}
	}

	private static String detectPlatform() {
		String os = System.getProperty("os.name").toLowerCase();
		String arch = System.getProperty("os.arch").toLowerCase();

		String osName;
		if (os.contains("windows")) {
			osName = "win32";
		} else if (os.contains("mac") || os.contains("darwin")) {
			osName = "darwin";
		} else if (os.contains("linux")) {
			osName = "linux";
		} else {
			throw new UnsatisfiedLinkError("Unsupported OS: " + os);
		}

		String archName;
		if (arch.contains("amd64") || arch.contains("x86_64")) {
			archName = "x86_64";
		} else if (arch.contains("aarch64") || arch.contains("arm64")) {
			archName = "aarch64";
		} else {
			throw new UnsatisfiedLinkError("Unsupported architecture: " + arch);
		}

		return osName + "-" + archName;
	}

	private static String getLibraryFileName() {
		String os = System.getProperty("os.name").toLowerCase();
		if (os.contains("windows")) {
			return LIBRARY_NAME + ".dll";
		} else if (os.contains("mac") || os.contains("darwin")) {
			return "lib" + LIBRARY_NAME + ".dylib";
		} else {
			return "lib" + LIBRARY_NAME + ".so";
		}
	}
}
