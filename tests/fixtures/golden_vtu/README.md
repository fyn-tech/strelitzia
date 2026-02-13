# Golden VTU Test Fixtures

This directory contains reference `.vtu` files for regression testing of the VTK writer.

## Purpose

Golden file tests ensure that the VTK output format remains stable across code changes. Any unintended modifications to the file format will be caught by comparing generated output against these reference files.

## Files

- `simple_scalar_ascii.vtu` - Minimal scalar field in ASCII format (2 points, temperature values 100, 200)
- `simple_vector_ascii.vtu` - Minimal vector field in ASCII format (2 points, unit vectors)

## Updating Golden Files

**Only update these files if you intentionally changed the VTK output format.**

### When to Update

- Changing VTK XML structure
- Modifying number formatting
- Altering whitespace or line breaks
- Updating VTK specification version

### How to Update

1. Run the golden file tests (they will fail):
   ```bash
   cargo test golden_file
   ```

2. Verify the new output is correct by:
   - Opening generated `.vtu` files in ParaView
   - Checking that data displays correctly
   - Confirming the change was intentional

3. Copy the new output to replace the golden file:
   ```bash
   cp test_golden_simple_scalar.vtu tests/fixtures/golden_vtu/simple_scalar_ascii.vtu
   ```

4. Re-run tests to confirm they pass:
   ```bash
   cargo test golden_file
   ```

5. Commit with descriptive message:
   ```bash
   git add tests/fixtures/golden_vtu/
   git commit -m "Update VTK golden files after [reason for change]"
   ```

## Test Philosophy

These tests complement semantic validation:
- **Semantic tests**: Verify correctness (field names, component counts, values)
- **Golden file tests**: Verify consistency (exact format preservation)

Both are valuable for different reasons!
