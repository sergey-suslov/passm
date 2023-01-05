# passm 🔒

<div align="center">
<img src="assets/logo_wide.svg" width="500px" alt="refinery Logo">
<p>
 <b>Self hosted password management simplified</b>
</p>
</div>
<br/>

**Passm strives to make self hosted password management as easy as possible.** 

Main idea of this project is to be as easy in use as centralised password managers (e.g. lastpass) by providing syncing capabilities to user owned storages. Passwords and encrypting keys are stored localy but can be exported and imorted from multiple providers (e.g. Dropbox, github). While passwords are encrypted by user's pgp private keys, private keys themselfs are encrypted with master password when exported.

<br/>

<hr/>
<div display="flex">
<img src="assets/demo-1.png" alt="refinery Logo">
<img src="assets/demo-2.png" alt="refinery Logo">
<img src="assets/demo-3.png" alt="refinery Logo">
</div>


## Run debug

`cargo run --bin passm`

## Roadmap

- [x] PGP key generation
- [x] Create/Edit/Delete you passwords
- [x] Export excrypted pgp secret key to local storage
- [ ] Sync passwords with a storage of you choise
    - [ ] Dropbox
    - [ ] ...
