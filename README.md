# [Thorium](https://github.com/Alex313031/thorium) Updater (Windows)

Based on [`template-rs`](https://github.com/just-shadyumbrella/template-rs#prerequisites).

## Usage

- `/help`, `/?`: Show this
- `/silent`: Do not show console output.
- `/repair`: Reinstall Thorium without uninstalling current.
  - `/force`: Force uninstall existing Thorium if any.
    - `/clearuserdata`: Clear Thorium user profile and data as well.
- `/beta`: By default the program will get release tagged as `latest`, switch this to get the most recent and available/match release.
- `/nocache`: By default the program will not delete downloaded installer after use as cache, switch this to disable.
- `/clearcache`: Only delete downloaded installer, nothing else. Useful if installer/cache corrupted.
- `/repo`: Override GitHub repository of Thorium update source. (Default: current active maintainer)
  > ```batch
  > /repo=Alex313031/Thorium-Win
  > ```
> [!WARNING]
> This may not future-proof, expect breaking changes in the future.

- `/simd`: Force select specific CPU instruction set build. (Default: automatically detect your maximum CPU capability)
  >  ```batch
  > /simd=AVX512
  > 
  > :: Show list
  > /simd=show
  > ```

- `/retry`: Retry download if failed. (Default: `3`)
  > ```batch
  > /retry=5
  > ```

> [!NOTE]
> You can append Chromium installer arguments to the end of the arguments.

## See for more

- [Bugs](../../issues?q=is%3Aissue+label%3Abug)
- [Feature requests](../../issues?q=is%3Aissue+label%3Aenhancement)
