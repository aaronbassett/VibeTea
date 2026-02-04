# GitHub Action Contract

**Feature Branch**: `004-monitor-gh-actions`
**Date**: 2026-02-03
**Priority**: P3 (Lower priority than core env var support)

## Composite Action: vibetea-monitor

### action.yml

```yaml
name: 'VibeTea Monitor'
description: 'Run VibeTea monitor to track Claude Code events in GitHub Actions'
author: 'VibeTea'

branding:
  icon: 'activity'
  color: 'green'

inputs:
  server-url:
    description: 'VibeTea server URL'
    required: true
  private-key:
    description: 'Base64-encoded Ed25519 private key seed'
    required: true
  source-id:
    description: 'Custom source identifier'
    required: false
    default: ''
  version:
    description: 'Monitor version to download'
    required: false
    default: 'latest'

outputs:
  monitor-pid:
    description: 'PID of the background monitor process'
    value: ${{ steps.start-monitor.outputs.pid }}

runs:
  using: 'composite'
  steps:
    - name: Download VibeTea Monitor
      id: download
      shell: bash
      run: |
        VERSION="${{ inputs.version }}"
        if [ "$VERSION" = "latest" ]; then
          URL="https://github.com/org/vibetea/releases/latest/download/vibetea-monitor-linux-amd64"
        else
          URL="https://github.com/org/vibetea/releases/download/$VERSION/vibetea-monitor-linux-amd64"
        fi
        curl -sSL "$URL" -o /tmp/vibetea-monitor
        chmod +x /tmp/vibetea-monitor
        echo "binary=/tmp/vibetea-monitor" >> $GITHUB_OUTPUT

    - name: Start Monitor
      id: start-monitor
      shell: bash
      env:
        VIBETEA_PRIVATE_KEY: ${{ inputs.private-key }}
        VIBETEA_SERVER_URL: ${{ inputs.server-url }}
        VIBETEA_SOURCE_ID: ${{ inputs.source-id || format('github-{0}-{1}', github.repository, github.run_id) }}
      run: |
        ${{ steps.download.outputs.binary }} run &
        PID=$!
        echo "pid=$PID" >> $GITHUB_OUTPUT
        # Wait briefly to verify it started
        sleep 1
        if ! kill -0 $PID 2>/dev/null; then
          echo "::error::VibeTea Monitor failed to start"
          exit 1
        fi
        echo "::notice::VibeTea Monitor started (PID: $PID)"
```

### Usage

#### Basic Usage

```yaml
- uses: org/vibetea/.github/actions/vibetea-monitor@v1
  with:
    server-url: ${{ vars.VIBETEA_SERVER_URL }}
    private-key: ${{ secrets.VIBETEA_PRIVATE_KEY }}
```

#### With Custom Source ID

```yaml
- uses: org/vibetea/.github/actions/vibetea-monitor@v1
  with:
    server-url: ${{ vars.VIBETEA_SERVER_URL }}
    private-key: ${{ secrets.VIBETEA_PRIVATE_KEY }}
    source-id: 'pr-${{ github.event.pull_request.number }}'
```

#### Pinned Version

```yaml
- uses: org/vibetea/.github/actions/vibetea-monitor@v1
  with:
    server-url: ${{ vars.VIBETEA_SERVER_URL }}
    private-key: ${{ secrets.VIBETEA_PRIVATE_KEY }}
    version: 'v1.2.3'
```

### Contract Tests

1. **action_starts_monitor_in_background**: Monitor process runs in background after action completes
2. **action_outputs_pid**: `monitor-pid` output contains valid PID
3. **action_fails_on_invalid_key**: Action fails with clear error if private-key is invalid
4. **action_uses_default_source_id**: Without source-id input, uses `github-{repo}-{run_id}` format
5. **action_custom_source_id**: Custom source-id is passed to monitor
6. **action_downloads_correct_version**: Version input controls download URL

### Inputs Contract

| Input | Type | Required | Validation |
|-------|------|----------|------------|
| `server-url` | String | Yes | Must be valid URL (https:// preferred) |
| `private-key` | String | Yes | Must be valid Base64, decode to 32 bytes |
| `source-id` | String | No | Any non-empty string; default is generated |
| `version` | String | No | 'latest' or semver like 'v1.2.3' |

### Outputs Contract

| Output | Type | Description |
|--------|------|-------------|
| `monitor-pid` | Number | Process ID of background monitor |

### Error Handling

| Condition | Behavior | User Message |
|-----------|----------|--------------|
| Download fails | Action fails | `::error::Failed to download VibeTea Monitor` |
| Monitor won't start | Action fails | `::error::VibeTea Monitor failed to start` |
| Invalid private-key | Monitor exits 1 | Error visible in logs from monitor |
| Server unreachable | Monitor logs warning | Non-blocking (FR-029) |

### Shutdown Behavior

- Monitor receives SIGTERM when job ends
- Attempts to flush buffered events (5 second timeout)
- Workflow succeeds regardless of event transmission (FR-029, FR-030)
- Exit code 0 even if some events couldn't be sent
