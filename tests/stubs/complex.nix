{
  derivations = {
    irrelevant_derivation_a = { irrelevant_secret }: ''
      irrelevant_secret=${irrelevant_secret}
    '';
    irrelevant_derivation_b = { foo, bar, irrelevant_secret }: ''
      irrelevant_secret=${irrelevant_secret}
      foo=${foo}
      bar=${bar}
    '';
    relevant_derivation_a = { foo }: ''
      foo=${foo}
    '';
    relevant_derivation_b = { bar }: ''
      foo=${bar}
    '';
    relevant_derivation_c = { foo, bar }: ''
      foo=${foo}
      bar=${bar}
    '';
  };

  secrets = {
    foo.hosts = [ "example" ];
    foo.enc = "foo";
    bar.hosts = [ "example" ];
    bar.enc = "bar";
    irrelevant_secret.hosts = [ ];
    irrelevant_secret.enc = "irrelevant";
  };
}