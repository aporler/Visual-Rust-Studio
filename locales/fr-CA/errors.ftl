# Visual Rust Studio — explications des diagnostics Rust (Français, Canada)
#
# Clés : rustc-<code d'erreur> (ex. : rustc-E0308) et lint-<nom du lint>.
# Chaque explication est associée à un code officiel : elle ne remplace jamais le
# message original du compilateur, qui reste affiché.

rustc-E0061 = La fonction est appelée avec un mauvais nombre d'arguments. Comparez l'appel avec la signature de la fonction (les paramètres entre parenthèses).
rustc-E0106 = Il manque une durée de vie (lifetime) à une référence (&). Quand une structure ou une fonction retourne une référence, Rust doit savoir combien de temps vivent les données référencées. Souvent, retourner une valeur possédée (String au lieu de &str) est plus simple.
rustc-E0277 = Un type n'implémente pas un trait nécessaire. Par exemple, on ne peut pas afficher avec {"{}"} un type qui n'implémente pas Display, ni additionner des valeurs de types différents. Vérifiez les types en cause ou implémentez/dérivez le trait (#[derive(Debug)], etc.).
rustc-E0308 = Types incompatibles : Rust attendait un type mais en a trouvé un autre. Par exemple, une String là où un nombre est attendu. Convertissez la valeur (parse(), to_string(), as) ou changez le type déclaré.
rustc-E0369 = L'opérateur (+, -, ==, …) ne peut pas être utilisé avec ces types. Les deux opérandes doivent être compatibles, ou le type doit implémenter le trait correspondant (Add, PartialEq…).
rustc-E0382 = Possession (ownership) : la valeur a été déplacée (donnée) à une autre variable ou fonction ; elle n'est donc plus utilisable ici. Utilisez une référence (&valeur), clonez-la (.clone()) ou réorganisez le code.
rustc-E0384 = Une variable déclarée sans mut ne peut pas être modifiée. Écrivez let mut nom = … si la valeur doit changer.
rustc-E0412 = Ce type est inconnu à cet endroit. Vérifiez son orthographe ou importez-le avec use (par exemple use std::collections::HashMap;).
rustc-E0425 = Ce nom (variable ou fonction) est inconnu ici. Vérifiez son orthographe et assurez-vous qu'il est déclaré avant son utilisation, dans la même portée (entre les mêmes accolades).
rustc-E0432 = L'importation (use) est introuvable. Vérifiez le chemin du module et que la dépendance est déclarée dans Cargo.toml.
rustc-E0433 = Rust ne peut pas résoudre ce chemin : il manque probablement une importation (use) ou une dépendance dans Cargo.toml.
rustc-E0499 = Emprunt : vous ne pouvez pas avoir deux références mutables (&mut) vers la même valeur en même temps. Terminez l'utilisation de la première avant de créer la seconde, ou séparez les données.
rustc-E0502 = Emprunt : la valeur est déjà empruntée en lecture (&) pendant que vous essayez de la modifier (&mut), ou l'inverse. Raccourcissez la durée de l'un des emprunts, par exemple en copiant d'abord la valeur nécessaire.
rustc-E0505 = Vous essayez de déplacer une valeur alors qu'elle est encore empruntée. Terminez d'abord l'utilisation de la référence, ou clonez la valeur.
rustc-E0596 = Vous essayez de modifier une valeur à travers une variable ou une référence non mutable. Ajoutez mut à la variable (let mut) ou prenez une référence mutable (&mut).
rustc-E0597 = La valeur ne vit pas assez longtemps : elle est détruite à la fin de son bloc alors qu'une référence vers elle est encore utilisée. Déplacez la déclaration dans un bloc englobant ou retournez une valeur possédée.
rustc-E0599 = Cette méthode n'existe pas pour ce type, ou le trait qui la fournit n'est pas importé. Vérifiez le type de la valeur et l'orthographe de la méthode.
lint-unused_variables = Cette variable n'est jamais utilisée. Supprimez-la, ou préfixez son nom par _ pour indiquer que c'est voulu.
lint-unused_imports = Cette importation (use) n'est pas utilisée. Vous pouvez la supprimer.
lint-unused_mut = Cette variable est déclarée mut mais n'est jamais modifiée. Vous pouvez retirer mut.
lint-dead_code = Ce code (fonction, champ, type) n'est jamais utilisé.
