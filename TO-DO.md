### First thing to do :  
        - Buffer (src/core/buffer) : stilau do `rope, piece-table & content` & nickels do : `memory & storage`
            ```
            C’est le cœur de l’éditeur : gérer le texte, l’insertion, la suppression, les structures type rope ou piece table, et la mémoire.
            Commence par définir les interfaces et traits (traits.rs) pour que tout reste modulaire.
            Assure la gestion de la mémoire et du stockage (pool, arena, slab).
            Sans buffer stable, rien d’autre ne peut vraiment fonctionner.
            ```

### Thing to do :
    - The [utils](src/utils/) folder for the basic & complex help