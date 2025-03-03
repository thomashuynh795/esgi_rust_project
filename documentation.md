# Implémentation et Avancées du Projet

## Algorithme de Navigation
Nous avons implémenté **l'algorithme de Trémaux** pour optimiser la sortie du labyrinthe. Cet algorithme nous permet de trouver le chemin un chemin le plus vierge possible en retenant les cases déjà explorées.

## Construction de la Grille Globale
Nous avons utilisé les données des **radars VIEW** pour reconstruire dynamiquement une **carte complète du labyrinthe**. En fonction de l'orientation du joueur et des informations reçues, nous avons progressivement révélé les murs et les passages, permettant une navigation plus précise.

## Développement d'un Mini-Serveur
Nous avons mis en place un **mini-serveur** permettant de tester notre client de manière autonome. Ce serveur :
- Simule le comportement du serveur officiel.
- Encode et décode les messages de la radar view et d'enregistrement.

## Gestion de l'Encodage et du Décodage
- Nous avons **décodé le Base64** des données reçues pour reconstruire les informations du labyrinthe.
- Nous avons également **encodé les données en Base64** via notre mini-serveur pour assurer la compatibilité avec le serveur officiel.
- Nous avons utilisé **Serde** pour la **sérialisation/désérialisation en JSON**, permettant un formatage conforme aux spécifications du projet.

## Gestion des Challenges
Notre client est capable de **relever les différents challenges** proposés par le serveur :
- Résolution des énigmes mathématiques comme `SecretSumModulo` mais la fonctionnalité reste instable.

## Système de Déblocage Dynamique
Nous avons développé un **système de débogage des fonctionnalités** à travers l'application en utilisant **une macro Rust**. Cette approche nous a permis d’activer/désactiver dynamiquement des fonctionnalités selon l’état du jeu et les besoins de la partie.

## Tests unitaires
Nous avons ajouté des **tests unitaires** pour s'assurer du bon comportement des fonctions.

## Communication Client-Serveur
- Notre **client communique correctement avec le serveur**, respectant le protocole de messages JSON spécifié dans le README.md.

## Multithreading et Gestion des Tours
Nous avons utilisé **des threads** pour la gestion du jeu, notamment pour :
- La gestion des **tours des joueurs** sans bloquer l’ensemble du programme.
