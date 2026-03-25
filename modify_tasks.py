import re

with open("src/pipeline/tasks.rs", "r") as f:
    text = f.read()

# Replace enums and impls
text = text.replace("pub enum ValidationTask {", "pub enum ValidationTask<'a> {")
text = text.replace("impl ValidationTask {", "impl<'a> ValidationTask<'a> {")

text = text.replace("pub enum ParseTask {", "pub enum ParseTask<'a> {")
text = text.replace("impl ParseTask {", "impl<'a> ParseTask<'a> {")

text = text.replace("pub enum ExecutionTask {", "pub enum ExecutionTask<'a> {")
text = text.replace("impl ExecutionTask {", "impl<'a> ExecutionTask<'a> {")

# Inside the enums, replace String with std::borrow::Cow<'a, str>
def replace_string_in_enum(match):
    enum_body = match.group(0)
    enum_body = re.sub(r":\s*String", r": std::borrow::Cow<'a, str>", enum_body)
    enum_body = enum_body.replace("Vec<String>", "Vec<std::borrow::Cow<'a, str>>")
    enum_body = enum_body.replace("std::sync::Arc<[ValueNode]>", "std::sync::Arc<[ValueNode<'a>]>")
    enum_body = enum_body.replace("std::sync::Arc<[(std::sync::Arc<str>, ValueNode)]>", "std::sync::Arc<[(std::borrow::Cow<'a, str>, ValueNode<'a>)]>")
    return enum_body

text = re.sub(r"pub enum ValidationTask<'a> \{.*?\n\}", replace_string_in_enum, text, flags=re.DOTALL)
text = re.sub(r"pub enum ParseTask<'a> \{.*?\n\}", replace_string_in_enum, text, flags=re.DOTALL)
text = re.sub(r"pub enum ExecutionTask<'a> \{.*?\n\}", replace_string_in_enum, text, flags=re.DOTALL)

with open("src/pipeline/tasks.rs", "w") as f:
    f.write(text)

