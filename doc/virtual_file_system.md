# Virtual File System

Specifications can create files to run tests against.
These files exist temporarily while the tests run.

You create a file using the `file` function:

```text,file(path="example.txt")
Example file content
```

The file path is then set in an environment variable which is available in future scripts.

```shell,script(name="cat-file")
cat "$FILE_EXAMPLE_TXT"
```

Will output:

```text,verify(script_name="cat-file", output=output)
Example file content
```