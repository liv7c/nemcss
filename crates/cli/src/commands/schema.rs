/// Prints the JSON schema for `NemCssConfig` to stdout.
pub fn schema() -> miette::Result<()> {
    let schema = config::generate_schema()?;
    print!("{}", schema);
    Ok(())
}
