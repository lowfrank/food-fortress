' https://www.winhelponline.com/blog/run-bat-files-invisibly-without-displaying-command-prompt/
Set WshShell = CreateObject("WScript.Shell") 
WshShell.Run chr(34) & "run.bat" & Chr(34), 0
Set WshShell = Nothing