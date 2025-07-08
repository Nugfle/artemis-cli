# ArtemisCLI

**A tool designed to make the lives of students easier when working with the online learning platform [Artemis](https://github.com/ls1intum/Artemis).**

ArtemisCLI gives you full control to start and manage exercises from your command line.

## Getting started

To get started you need to have [rust](https://www.rust-lang.org/) installed. You can then install the tool simply by running 
```
cargo install artemis-cli
```

## Setting up Authentication

To set up your login information simply run 
```
artemis-cli config [YOUR USERNAME] [YOUR PASSWORD]
```

## Working with ArtemisCLI

You can list all enlisted courses by running 
```
artemis-cli list-courses
```
and all tasks in a course with 
```
artemis-cli list-tasks [COURSE ID]
```
You can then start a task which automaticly clones the repository by running
```
artemis-cli start-task [TASK ID]
```
If you are finished and want to see the tests, you can run 
```
artemis-cli submit [TASK ID]
```
to automacily create a commit, push to the remote repository and fetch the updated test results for you.

## Contributing

Feel free to contribute to this project, please note that this is a sideproject built out of annoyance with the current system and I wont be supporting it for longer than neccesary.
