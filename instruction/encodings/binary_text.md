# Spécifications encodage textuel de données binaires b64

## L'encodage binaire -> texte

### Entrée

- Données binaires brutes (tableau d'octets)

### Sortie

- Chaîne de caractères encodée (cette fonction ne peut pas échouer)

### Comportement

- Convertit les données binaires en représentation textuelle b64
- Traite les données par groupes de 3 octets
- Produit jusqu'à 4 caractères pour chaque groupe de 3 octets.

    * Si la longueur des données d'entrée n'est pas un multiple de 3,
      elles sont éventuellement complétées avec des `0`.

    * Le premier octet produit tout le temps 2 caractères

    * Le 3ème caractère est effectivement produit que s'il y a bien 2 octets (hors padding) valides dans le groupe
      courant.

    * Le 4ème caractère est effectivement produit que s'il y a bien 3 octets (hors padding) valides dans le groupe
      courant.

- Utilise l'alphabet pour chaque paquet de 6 bits:
    * a-z (26 caractères, encodant les valeurs de 0 à 25)
    * A-Z (26 caractères, encodant les valeurs de 26 à 51)
    * 0-9 (10 caractères, encodant les valeurs de 52 à 61)
    * \+ et / (2 caractères encodant les valeurs 62 et 63)

  ```
  |01234567|01234567|01234567| : 3 octets d'entrée
  |012345|670123|456701|234567| : 4 caractères encodés sur 6 bits
  ```

## Le décodage

### Entrée

- Chaîne de caractères encodée en b64

### Sortie

- En cas de succès : données binaires décodées
- En cas d'erreur : message d'erreur explicitant le problème d'encodage

### Comportement

- Convertit une représentation b64 en données binaires originales
- Un padding de 0 (éventuellement virtuel, non matérialisé dans le stockage) complète si nécessaire la chaîne de
  caractères pour atteindre une taille qui soit un multiple de 4 afin d'appliquer l'algorithme par paquet de 4 caractères.

### Erreurs possibles

- Taille invalide (la seule taille invalide est de la forme 4n+1)
- Caractères non autorisés

## Examples:

````
assert_eq!(encode(&[0]), "aa");
assert_eq!(encode(&[25]), "gq");
assert_eq!(encode(&[26]), "gG");
assert_eq!(encode(&[51]), "mW");
assert_eq!(encode(&[52]), "na");
assert_eq!(encode(&[61]), "pq");
assert_eq!(encode(&[62]), "pG");
assert_eq!(encode(&[63]), "pW");
assert_eq!(encode(b"Hello, World!"), "sgvSBg8SifDVCMXKiq");
assert_eq!(encode(&[0,1,2,3...254,255]), "aaecaWqfbGCicqOlda0odXareHmufryxgbKAgXWDhH8GisiJjcuMjYGPkISSls4VmdeYmZq1nJC4otO7pd0+p0bbqKneruzhseLks0XntK9quvjtvfvwv1HzwLTCxv5FygfIy2rLzMDOAwPRBg1UB3bXCNn0Dxz3EhL6E3X9FN+aGykdHiwgH4IjIOUmJy6pKjgsK5svLPEyMzQBNj2EN6cHOQoKPAANQkMQQ6YTRQ+WSBkZTlw2T7I5URU8VB6/WmhcW8tfXSFiYCRlZm3oZ9dr0Tpu1DBx2nNA29ZD3T/G4ElJ5oxM5+JP6UVS7E7V8phY8/t19VF4+FR7/p3+/W");
```