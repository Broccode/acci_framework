# Setting Up Git Pre-Commit Hook

This document explains how to set up the Git pre-commit hook in our repository so that the defined checks are automatically executed before each commit.

## Steps

1. **Navigate to the Repository Root**

   Open your terminal and change to the root directory of your repository:

   ```bash
   cd /path/to/your/repository
   ```

2. **Create the Pre-Commit Hook**

   Git looks for hook scripts in the `.git/hooks` directory. Create or edit the `pre-commit` file:

   ```bash
   cd .git/hooks
   touch pre-commit
   ```

3. **Add the Hook Script Content**

   Open the `pre-commit` file in your preferred text editor and add the following content:

   ```bash
   #!/usr/bin/env bash
   exec "$(git rev-parse --show-toplevel)/scripts/pre-commit.sh"
   ```

   This command ensures that when running `git commit`, the script located at `scripts/pre-commit.sh` is executed.

4. **Make the Hook Executable**

   Set the correct permissions for the pre-commit hook:

   ```bash
   chmod +x .git/hooks/pre-commit
   ```

5. **Test the Hook**

   Try making a commit to verify that the hook runs correctly. If any pre-commit check (such as formatting, fixes, or Clippy checks) fails, the commit will be aborted.

## Notes

- The script `scripts/pre-commit.sh` calls the `prepare-commit` target defined in the Makefile, which in turn runs code formatting, code fixes, and Clippy.
- All developers should set up this pre-commit hook locally to maintain consistent code quality.
- Any changes to the pre-commit process should be discussed and aligned with the team.
