# Food Fortress: a fridge manager written in Rust.

The Food Fortress is a very simple manager of fridge inventory. You can record your own food items, set a best before date, and the app will remember your entries. Each food has a color based on how close to today the best before date is.
You can add multiple copies of the same food at once, and the simple and intuitive GUI helps you in deciding what you should eat today!

To complile Food Fortress, you must have Rust 1.65 or higher, and use the `+stable` command:
```
cargo +stable b --release
```

In the `run` folder, you can find a `launch_bat.vbs` and a `run.bat`. I recommend creating a shortcut to the Dektop of the `launch_bat.vbs`, renaming it and setting the icon (located in `images/refrigerator.ico`). This way you can have a good looking shortcut in your desktop.

Note that Food Fortress is only available for the Windows environment.
