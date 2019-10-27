## mkrepo

Create directory and `git init` and initial commit in imitation of [ghq](https://github.com/motemen/ghq)'s management directory structure.

### Installation

```
cargo install mkrepo
```

### Usage

`mkrepo` requires following `.gitconfig` values.

- `ghq.root`
- `mkrepo.service`
- `mkrepo.username` or `user.name`

Add these values to your `~/.gitconfig`.

```
[user]
name="himanoa"
[ghq]
root="~/src"
[mkrepo]
service="github.com"
username="himanoa"
```

#### Simple

```
$ mkrepo sample-repository
$ ls -al ~/src/github.com/himanoa/sample-repository
./ ../ .git/
```

#### Overwrite author name

```
$ mkrepo -a himanoa-sandbox sample-repository
$ ls -al ~/src/github.com/himanoa-sandbox/sample-repository
./ ../ .git/
```

#### Overwrite service name

```
$ mkrepo -s example.com sample-repository
$ ls -al ~/src/example.com/himanoa/sample-repository
./ ../ .git/
```

#### Overwrite first commit message

```
$ mkrepo -m "Custom initial commit message" sample-repository
$ cd ~/src/github.com/himanoa/sample-repository
$ git show

commit 838a05bebd96e04a21d539946c92f78f9eb233d0 (HEAD -> master)
Author: himanoa <matsunoappy@gmail.com>
Date:   Fri Oct 25 05:20:10 2019 +0900

    Custom initial commit message
```

### Author

- [himanoa](https://twitter.com/h1manoa)

### LICENSE

MIT
