# Contributing / Contribuer

*[English](#english) · [Français](#français)*

---

## English

Thank you for your interest in Visual Rust Studio (VRS).

### Before submitting

- `cargo fmt --all`, `cargo clippy --workspace --all-targets` and
  `cargo test --workspace` must pass without errors or warnings.
- Every user-visible text goes through `vrs-i18n` (keys in `locales/`); fr-CA
  and en-US must stay complete (`cargo run -p vrs-i18n --bin vrs-i18n-check`).
- Documentation is written in English first, then in French.

### Contributor License Agreement (CLA)

Visual Rust Studio is distributed under the GNU AGPL-3.0. Its copyright holder,
aporler (the "Maintainer"), also reserves the right to distribute VRS under
other terms. To keep this possible, every contribution requires accepting the
following agreement. Opening a pull request means you accept it.

1. **Definitions.** "Contribution" means any code, documentation, translation
   or other material you submit to the project, in any form.
2. **Originality.** You are the author of your Contribution, or you have the
   right to submit it under this agreement. If your employer has rights to it,
   you have obtained their permission.
3. **Copyright license.** You grant the Maintainer a perpetual, worldwide,
   non-exclusive, free of charge, irrevocable license to use, reproduce,
   modify, publicly display, distribute, sublicense and relicense your
   Contribution, under any license, including proprietary and commercial
   licenses.
4. **Patent license.** You grant the Maintainer and recipients of VRS a
   perpetual, worldwide, non-exclusive, free of charge, irrevocable patent
   license for your Contribution, as part of VRS.
5. **Your rights.** You keep the copyright on your Contribution and may use it
   for any other purpose.
6. **Public license.** Your Contribution is distributed to the public under the
   project's license: AGPL-3.0-only for Visual Rust Studio, MIT-0 for
   `crates/vrs-runtime` and generated code.
7. **No warranty.** Your Contribution is provided "as is", without warranty of
   any kind.

---

## Français

Merci de votre intérêt pour Visual Rust Studio (VRS).

### Avant de soumettre

- `cargo fmt --all`, `cargo clippy --workspace --all-targets` et
  `cargo test --workspace` doivent passer sans erreur ni avertissement.
- Tout texte visible passe par `vrs-i18n` (clés dans `locales/`) ; fr-CA et
  en-US doivent rester complets (`cargo run -p vrs-i18n --bin vrs-i18n-check`).
- La documentation est rédigée en anglais d'abord, puis en français.

### Accord de licence de contribution (CLA)

Visual Rust Studio est distribué sous licence GNU AGPL-3.0. Son titulaire des
droits, aporler (le « Mainteneur »), se réserve aussi le droit de distribuer VRS
sous d'autres conditions. Pour que cela reste possible, toute contribution
nécessite l'acceptation de l'accord suivant. Ouvrir une demande de fusion
(pull request) vaut acceptation.

1. **Définitions.** « Contribution » désigne tout code, documentation,
   traduction ou autre élément que vous soumettez au projet, sous quelque forme
   que ce soit.
2. **Originalité.** Vous êtes l'auteur de votre Contribution, ou vous avez le
   droit de la soumettre selon cet accord. Si votre employeur détient des
   droits sur celle-ci, vous avez obtenu son autorisation.
3. **Licence de droit d'auteur.** Vous accordez au Mainteneur une licence
   perpétuelle, mondiale, non exclusive, gratuite et irrévocable d'utiliser,
   reproduire, modifier, afficher publiquement, distribuer, sous-licencier et
   relicencier votre Contribution, sous toute licence, y compris propriétaire
   et commerciale.
4. **Licence de brevets.** Vous accordez au Mainteneur et aux destinataires de
   VRS une licence de brevets perpétuelle, mondiale, non exclusive, gratuite et
   irrévocable pour votre Contribution, en tant que partie de VRS.
5. **Vos droits.** Vous conservez le droit d'auteur sur votre Contribution et
   pouvez l'utiliser à toute autre fin.
6. **Licence publique.** Votre Contribution est distribuée au public sous la
   licence du projet : AGPL-3.0-only pour Visual Rust Studio, MIT-0 pour
   `crates/vrs-runtime` et le code généré.
7. **Aucune garantie.** Votre Contribution est fournie « telle quelle », sans
   garantie d'aucune sorte.
