# Set Me Up!

Set Me Up! (SMU) is a simple implementation of an Ansible-based provisioning server.
Its main goal is to require very little from the clients we are trying to provision.
As a matter of fact, all they need is an SSH server and client.

The idea is:

1. The client opens an SSH connection to the provisioning SMU server.
2. That server prompts for some information.
3. The client gets automatically provisioned!


## Usage

On the client you would like to provision, simply connect to the SMU server with:

	$ ssh -R 0:localhost:22 smu@smu-server.tld
	Allocated port 44561 for remote forward to localhost:22
	Welcome to Set Me Up!

The `-R` option is what allows SMU to tunnel back to your system for provisioning.
For the rest of this to work, you will then be guided through a little wizard:

1. First, you will need to tell SMU which port was allocated for reverse tunnelling.
   In the example above, this was 44561.
2. Then, SMU will ask for a username to use on your machine.
   Depending on what you are provisioning, you might need to provide a sudoer here.
3. Finally, SMU will send you the public end of an ECDSA key pair.
   You will need to ensure this key pair can be used to login to your machine with the username provided above.
   Typically, this means adding the public key to that user's `~/.ssh/authorized_keys`.

When this is done, SMU will test it all to make sure it can reach your system.
If all goes well, you will then get a prompt through which you can configure your provisioning.
This is where you pick which Ansible playbook should be played on your machine.
Which playbooks are available depends on the configuration on the SMU server.

> Note that here, Set Me Up! is configured as the shell for the smu user, which makes the command a little shorter.
> If your clients already have accounts on your server (and SSH access), there are other options.


## Base server setup

Let's go through installing and configuring Set Me Up!

### Installation

Set Me Up! is written in Rust. Downloading, compiling, testing and installing go like this:

	$ git clone https://gitlab.com/julienjpk/setmeup
	$ cd setmeup
	$ cargo test
	$ cargo build --release
	$ cargo install

### Setting up client access

How you set up SMU on your server is really up to you.
If your clients already have accounts on your server, they may use those to run SMU:

	$ ssh -R 0:localhost:22 bob@smu-server.tld setmeup

**In that scenario, no further setup is required for client access.**

If that is not the case, you may create an `smu` user for your clients to share.

	# useradd -md /var/lib/setmeup -s /usr/local/bin/setmeup

The `-s` option sets the shell to the SMU binary, which means people logging in as `smu` will start SMU by default.
The `smu` user also needs to have a home directory (`/var/lib/setmeup` here).

> The user does not need to be called `smu`, if you don't like it. In fact, you can also create several users and set them up independently.

You may then grant access to the `smu` user as you would to any other user (`/var/lib/setmeup/.ssh/authorized_keys` or `passwd smu`).

### Restrict the smu user

If you'd like to force the `smu` user to use SMU and not run anything else on your system, I suggest adding this to your SSH server configuration:

	# /etc/ssh/sshd_config

	Match User smu
	    ForceCommand /usr/local/bin/setmeup

Remember to reload your SSH configuration afterwards:

	$ systemctl reload sshd


## Configure provisioning sources and options

Set Me Up! looks for its configuration file in those locations, in order:

1. The path given through the `-c` switch
2. The `SETMEUP_CONF` environment variable.
3. `$XDG_CONFIG_DIR/setmeup/setmeup.toml`, relative to whoever the client is logged-in as.
4. `~/.setmeup.toml`, again, user-dependent.
5. `/etc/setmeup/setmeup.toml`
6. `/etc/setmeup.toml`

Options 1, 3 and 4 make it possible for each user to have its own SMU configuration. Note that as per specification, `XDG_CONFIG_DIR` defaults to `~/.config`.
They also allow you to configure SMU in `/var/lib/setmeup`, if that's what you chose earlier.
Option 2 is useful if you'd like to configure SMU through `/etc/environment`, `~/.pam_environment` or `~/.ssh/environment`.
The rest are what I believe to be sensible defaults.

The configuration file itself is [YAML](https://yaml.org/).

	sources:
	  some_local_source:
	    path: "/etc/setmeup/playbooks"
        playbook_match: "^public/.+\.ya?ml$"

	  some_git_repository:
	    path: "~/some_git_repository"
        recurse: yes
        pre_provision: "git pull"
        ansible_playbook:
	      path "/usr/local/bin/ansible-playbook"
	      args: ["--vault-password-file", "secrets.vault"]
	      env:
	        - name: "ANSIBLE_CONFIG"
	          value: "ansible.cfg"
	        - name: "ANSIBLE_ROLES_PATH"
	          value: "roles"

SMU looks for Ansible playbooks (`\.ya?ml$`) in each source's top-level directory without recursing, unless `recurse` is set.
The `playbook_match` setting can be used to set a different REGEX if necessary.
The REGEX is matched against the file path relative to the source's root, which means you can match through subdirectories.
You may also use the `ansible_playbook` dictionary to customise how `ansible-playbook` will be called for each source.
Note that SMU will always run `ansible-playbook` from your sources' root directories.
Finally, the `pre_provision` parameter can be set to have a command run before provisioning a client.
This is useful if your source is a git repository and you'd like it updated before your playbooks are looked up.


## About

Set Me Up! is a little project I came up with after I got annoyed of never having my dotfiles synced between all my machines.
There are great options for dotfiles management out there, but I felt like writing a little something to leverage Ansible's capabilities.
It also enables me to quickly set up some basic server apps wherever I can't be reached over SSH directly.
*(there probably exist a handful of nice Ansible provisoning servers too... but shush)*

Set Me Up! is licensed under the terms of the AGPL.
