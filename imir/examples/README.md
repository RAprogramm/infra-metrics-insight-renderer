# Configuration Examples

This directory contains example YAML configurations for `imir`.

## Available Examples

### [minimal.yaml](minimal.yaml)

Bare minimum configuration with only required fields.

**Use case**: Quick start, simple projects

```yaml
targets:
  - owner: octocat
    repository: hello-world
    type: open_source
```

**Run**:
```bash
imir targets --config examples/minimal.yaml --pretty
```

### [full-featured.yaml](full-featured.yaml)

Comprehensive configuration showcasing all available options.

**Use case**: Learning all features, complex setups

**Features demonstrated**:
- Profile dashboards
- Open source repositories
- Private projects
- Custom badge styling
- Multiple branches
- Time zone configurations
- All widget customization options

**Run**:
```bash
imir targets --config examples/full-featured.yaml --pretty
```

### [multi-repo.yaml](multi-repo.yaml)

Organization-wide multi-repository configuration.

**Use case**: Enterprise/team setups with many repositories

**Features demonstrated**:
- Frontend repositories
- Backend services
- Infrastructure code
- Documentation sites
- Team member profiles
- Consistent badge styling across organization

**Run**:
```bash
imir targets --config examples/multi-repo.yaml --pretty
```

## Testing Examples

Validate example configurations:

```bash
# Test all examples
for file in examples/*.yaml; do
    echo "Testing $file..."
    imir targets --config "$file" --pretty > /dev/null
    if [ $? -eq 0 ]; then
        echo "✓ $file is valid"
    else
        echo "✗ $file has errors"
    fi
done
```

## Creating Your Own Configuration

1. **Start with minimal**:
   ```bash
   cp examples/minimal.yaml my-config.yaml
   ```

2. **Add targets**:
   ```yaml
   targets:
     - owner: your-username
       repository: your-repo
       type: open_source
   ```

3. **Validate**:
   ```bash
   imir targets --config my-config.yaml --pretty
   ```

4. **Generate badges**:
   ```bash
   imir badge generate-all --config my-config.yaml --output metrics/
   ```

## Common Patterns

### Profile Dashboard

```yaml
- owner: username
  type: profile
  slug: my-profile
  display_name: My Developer Profile
  badge:
    style: for_the_badge
```

### Open Source Project

```yaml
- owner: organization
  repository: project-name
  type: open_source
  branch_name: main
  contributors_branch: main
  badge:
    style: classic
    widget:
      columns: 2
      alignment: center
```

### Private Repository

```yaml
- owner: company
  repository: internal-api
  type: private_project
  include_private: true
  badge:
    style: flat
```

### Multiple Branches

```yaml
# Production branch
- owner: myorg
  repository: webapp
  type: open_source
  slug: webapp-prod
  branch_name: main
  display_name: WebApp (Production)

# Staging branch
- owner: myorg
  repository: webapp
  type: open_source
  slug: webapp-staging
  branch_name: staging
  display_name: WebApp (Staging)
```

## Field Reference

### Required Fields

| Field | Type | Values | Description |
|-------|------|--------|-------------|
| `owner` | string | - | GitHub username or organization |
| `type` | string | `profile`, `open_source`, `private_project` | Target type |

### Conditional Fields

| Field | Required For | Description |
|-------|-------------|-------------|
| `repository` | `open_source`, `private_project` | Repository name |

### Optional Fields

| Field | Default | Description |
|-------|---------|-------------|
| `slug` | auto-generated | URL-safe identifier |
| `display_name` | auto-generated | Human-readable name |
| `branch_name` | `main` | Git branch to use |
| `contributors_branch` | `main` | Branch for contributors data |
| `target_path` | auto-generated | Output SVG path |
| `temp_artifact` | auto-generated | Temporary file path |
| `time_zone` | `UTC` | Time zone for metrics |
| `include_private` | `false` | Include private repos in profile |
| `badge.style` | `classic` | Badge style |
| `badge.widget.columns` | `2` | Widget columns |
| `badge.widget.alignment` | `center` | Widget alignment |
| `badge.widget.border_radius` | `6` | Border radius in pixels |

### Badge Styles

- `classic` - Traditional GitHub style
- `flat` - Minimalist flat design
- `for_the_badge` - Bold with uppercase text

### Widget Alignment

- `start` - Left-aligned
- `center` - Centered (default)
- `end` - Right-aligned

## Tips

1. **Use comments** for complex configurations:
   ```yaml
   # Backend services
   - owner: myorg
     repository: api-gateway
     type: private_project
   ```

2. **Keep slugs unique** to avoid conflicts:
   ```yaml
   slug: acme-webapp-main  # Not just "webapp"
   ```

3. **Group related targets** for organization:
   ```yaml
   # Frontend
   - owner: org
     repository: web-client

   # Backend
   - owner: org
     repository: api
   ```

4. **Test configurations** before deploying:
   ```bash
   imir targets --config my-config.yaml
   ```

## Further Reading

- [CONFIGURATION.md](../CONFIGURATION.md) - Detailed configuration reference
- [TROUBLESHOOTING.md](../TROUBLESHOOTING.md) - Common issues and solutions
- [PERFORMANCE.md](../PERFORMANCE.md) - Performance tuning guide
