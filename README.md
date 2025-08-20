# ArtemisCLI

**A tool designed to make the lives of students easier when working with the online learning platform [Artemis](https://github.com/ls1intum/Artemis).**

ArtemisCLI gives you full control to start and manage exercises from your command line. It has currently only been tested on Linux and with the Artemis server hosted by TU Dresden.


## Getting started


To get started you need to have [rust](https://www.rust-lang.org/) installed. You can then install the tool simply by running 
```
cargo install artemis-cli
```

## Setup

Make sure your ssh-agent is configured and running:
```
ssh-add -l
```

To set up your login information simply run 
```
artemis-cli config username [YOUR USERNAME]
artemis-cli config password [YOUR PASSWORD]
```
To configure the base url of the Artemis server run:
```
artemis-cli config base-url [BASE URL]
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
If you are finished and want to submit it run:
```
artemis-cli submit
```
To view the most recent test results run:
```
artemis-cli fetch [TASK ID]
```
to automacily create a commit, push to the remote repository and fetch the updated test results for you.

## Development

**This project is officialy archieved and there will be no further development done**
