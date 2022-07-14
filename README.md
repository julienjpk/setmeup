<img src="https://jjpk.me/uploads/8186f022fbe440cb8f3fbfe88cf97ff0.svg" align="center" />


# Set Me Up!

Set Me Up! is an Ansible-based provisioning server, allowing you to run playbooks on your local machine from a remote server.

1. You SSH into the provisioning server
2. You provide a username and register a public key on your local machine
3. You select a playbook interactively
3. That playbook is run on your machine

A typical use case is to provision a fresh setup without having to install Ansible, download your playbooks and so on. It's also quite convenient when the target machine cannot easily be reached over SSH.

## ⌨ Usage

On the client you would like to provision, simply connect to your SMU server with:

	$ ssh -TR 0:*:22 smu@setmeup.tld
	Allocated port 44561 for remote forward to *:22
	Welcome to Set Me Up!

Then follow the instructions :-)

> The `-R` option is what allows your SMU server to hop back to your machine and run your playbook: SSH reverse tunnelling. For more information, have a look at [the SSH client man page](https://linux.die.net/man/1/ssh).


## 🛠 Server installation

First, you'll need to make sure that your Ansible install comes with the `ansible.posix` collection. This can be checked using `ansible-galaxy collection list`. If you do not see it appear there, you can install it with:

    $ ansible-galaxy collection install ansible.posix

This might require root privileges (`sudo`) if you have installed Ansible system-wide (eg. with your package manager or `sudo pip`).

Then, you'll have to install the Set Me Up! executable on your server. The easiest approach is to download the latest binary [from GitLab CI](https://gitlab.com/julienjpk/setmeup/-/releases). To save yourself the trouble of glibc dependencies, go for the musl build and install it with:

    $ sudo cp setmeup_x86_64_musl-* /usr/local/bin/setmeup
	$ sudo chown root:root /usr/local/bin/setmeup
	$ sudo chmod 0755 /usr/local/bin/setmeup

You can also get it from [crates.io](https://crates.io/crates/setmeup) with Cargo:

	$ cargo install setmeup
    $ sudo install ${CARGO_HOME:-~/.cargo}/bin/setmeup /usr/local/bin

If you'd rather build it from source, you'll need a Rust toolchain. Compile and install with:

	$ git clone https://gitlab.com/julienjpk/setmeup
	$ cd setmeup
	$ cargo build --release
	$ sudo install target/release/setmeup /usr/local/bin

Next, allow the use of Set Me Up! as a shell. This makes the client command a little bit quicker to type.

    $ echo /usr/local/bin/setmeup | sudo tee -a /etc/shells

Create a user for Set Me Up! connections and make it use SMU as a shell:

    $ sudo useradd -md /var/lib/setmeup -s /usr/local/bin/setmeup smu

For extra safety, you may force the use of SMU for this user. This is done in your SSH server configuration file:

	# /etc/ssh/sshd_config

	Match User smu
	    ForceCommand /usr/local/bin/setmeup

Remember to reload your SSH configuration afterwards:

	$ sudo systemctl reload sshd


## ⚙ Configure provisioning sources and options

Set Me Up! looks for its configuration file in those locations, in order:

1. The path given through the `-c` switch
2. The *SETMEUP_CONF* environment variable.
3. *$XDG_CONFIG_DIR/setmeup/setmeup.yml*, relative to whoever the client is logged-in as (smu).
4. *~/.setmeup.yml*, again, user-dependent.
5. */etc/setmeup/setmeup.yml*
6. */etc/setmeup.yml*

The configuration file itself is [YAML](https://yaml.org/). For the server setup above, a good location for it is */var/lib/setmeup/.config/setmeup/setmeup.yml*).

	sources:
	  some_local_source:
	    path: "/etc/setmeup/playbooks"
        playbook_match: "^public/.+\.ya?ml$"

	  some_git_repository:
	    path: "~/some_git_repository"
        recurse: yes
        pre_provision: "git pull"
        ansible_playbook:
	      path: "/usr/local/bin/ansible-playbook"
	      env:
	        - name: "ANSIBLE_CONFIG"
	          value: "ansible.cfg"
	        - name: "ANSIBLE_ROLES_PATH"
	          value: "roles"

Here's what you need to know:

- Set Me Up! looks for Ansible playbooks (`\.ya?ml$`) in each source's top-level directory without recursing, unless `recurse` is set.
- The `playbook_match` setting can be used to set a different REGEX if necessary.
- The REGEX is matched against the file path relative to the source's root, which means you can match through subdirectories.
- You may also use the `ansible_playbook` dictionary to customise how `ansible-playbook` will be called for each source.
- Set Me Up! will always run `ansible-playbook` from your sources' root directories.
- The `pre_provision` parameter can be set to have a command run before provisioning a client. This is useful if your source is a git repository and you'd like it updated before your playbooks are looked up.


## About

Set Me Up! is a little project I came up with after I got annoyed of never having my dotfiles synced between all my machines.
There are great options for dotfiles management out there, but I felt like writing a little something to leverage Ansible's capabilities.
It also enables me to quickly set up some basic server apps wherever I can't be reached over SSH directly.
*(there probably exist a handful of nice Ansible provisoning servers too... but shush)*

Set Me Up! is licensed under the terms of the AGPL.
