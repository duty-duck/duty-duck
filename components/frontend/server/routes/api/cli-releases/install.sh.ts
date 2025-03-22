import { listReleases, Release } from ".";

const installScript = (release: Release) =>
    `#!/bin/sh

# This script will install the DutyDuck CLI tool, version ${release.name} in ~/.dutyduck/bin/
# and add it to your PATH.

TARGET_DIR=~/.dutyduck/bin/
TARGET_FILE=\${TARGET_DIR}dutyduck

# Determine the appropriate platform (linux-amd64, linux-arm64, macos-amd64, macos-arm64, windows-amd64)
ARCH=\$(uname -m)
if [ "\$ARCH" = "x86_64" ]; then
    ARCH="amd64"
elif [ "\$ARCH" = "aarch64" ]; then
    ARCH="arm64"
else
    echo "Unsupported architecture: \$ARCH"
    exit 1
fi

OS=\$(uname -s)
if [ "\$OS" = "Linux" ]; then
    OS="linux"
elif [ "\$OS" = "Darwin" ]; then
    OS="macos"
elif [ "\$OS" = "MINGW64_NT-10.0" ]; then
    OS="windows"
else
    echo "Unsupported OS: \$OS"
    exit 1
fi

PLATFORM="\${OS}-\${ARCH}"

# Prompt the user to confirm the installation
echo "This script will download the DutyDuck CLI tool, version ${release.name} in \${TARGET_DIR}. Do you want to continue? (y/n)"

read -r confirm
if [ "\$confirm" != "y" ]; then
    echo "Installation cancelled."
    exit 1
fi

# Create the target directory if it doesn't exist
mkdir -p \${TARGET_DIR}

# Download the appropriate binary
${release.platforms.map((platform) => {
        return `if [ "\$PLATFORM" = "${platform.platform}" ]; then
        curl -L -o \${TARGET_DIR}/dutyduck ${platform.url}
        fi`
    }).join("\n")
    }

if [ ! -f \${TARGET_DIR}/dutyduck ]; then
    echo "Failed to download the DutyDuck CLI tool for platform \${PLATFORM}"
    exit 1
fi

# Make the binary executable
chmod +x \${TARGET_DIR}/dutyduck

echo "DutyDuck CLI tool downloaded successfully in \${TARGET_DIR}."

echo "Do you want to add the DutyDuck CLI tool to your PATH? (y/n)"
read -r add_to_path
if [ "\$add_to_path" = "y" ]; then
    if [ -f ~/.zshrc ]; then
        echo "# Add DutyDuck CLI tool to PATH" >> ~/.zshrc
        echo "export PATH=\\$PATH:\${TARGET_DIR}" >> ~/.zshrc
    fi

    if [ -f ~/.bashrc ]; then 
        echo "# Add DutyDuck CLI tool to PATH" >> ~/.bashrc
        echo "export PATH=\\$PATH:\${TARGET_DIR}" >> ~/.bashrc
    fi

    if [ -f ~/.bash_profile ]; then
        echo "# Add DutyDuck CLI tool to PATH" >> ~/.bash_profile
        echo "export PATH=\\$PATH:\${TARGET_DIR}" >> ~/.bash_profile
    fi
    echo "DutyDuck CLI tool added to PATH. Please restart your shell or run 'source ~/.bashrc' or 'source ~/.zshrc' to apply the changes."
else
    echo "DutyDuck CLI tool not added to PATH. You can add it manually by adding the following line to your shell profile (e.g. ~/.bashrc, ~/.zshrc) and restarting your shell:"
    echo "export PATH=\$PATH:\${TARGET_DIR}"
fi

echo "Goodbye!"
exit 0
`


export default defineEventHandler(async event => {
    const releases = await listReleases(event);
    if (!releases.releases.length) {
        throw createError({
            statusCode: 404,
            statusMessage: "No releases found",
        });
    }
    const latestRelease = releases.releases[0];
    return installScript(latestRelease);
});