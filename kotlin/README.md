Generate reference colors files with:

```bash
kotlinc (Get-ChildItem -Path . -Recurse -Filter *.kt).FullName -include-runtime -d mcu.jar; java -jar mcu.jar
```