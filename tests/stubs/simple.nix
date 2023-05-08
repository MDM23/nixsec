{
  derivations = {
    irrelevant_derivation = { irrelevant_secret }: ''
      irrelevant_derivation -> ${irrelevant_secret}
    '';
  };

  secrets = {
    foo.hosts = [ "example" ];
    foo.enc = "foo";
    irrelevant_secret.hosts = [ ];
    irrelevant_secret.enc = "irrelevant";
  };
}