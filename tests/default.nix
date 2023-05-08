{ lib, self, ... }:
let
  mkSecrets = self.outputs.lib.mkSecrets;
  pkgs.writeShellScript = name: content: "/foo/scripts/${name}";
  pkgs.nix = "/foo";
  pkgs.age = "/foo";
  config.networking.hostName = "example";
in let
  empty = mkSecrets ./stubs/empty.nix { inherit config lib pkgs; };
  simple = mkSecrets ./stubs/simple.nix { inherit config lib pkgs; };
  complex = mkSecrets ./stubs/complex.nix { inherit config lib pkgs; };
in lib.debug.runTests {

  testEnvEmpty = {
    expr = empty.config.environment.etc;
    expected = { };
  };

  testConfigEmpty = {
    expr = builtins.hasAttr "secrets" empty.config;
    expected = false;
  };

  testEnvSimple = {
    expr = simple.config.environment.etc;
    expected = {
      "nixsec/secrets/foo" = {
        mode = "0400";
        text = "foo";
      };
    };
  };

  testConfigSimple = {
    expr = builtins.attrNames simple.options.secrets;
    expected = [ "foo" ];
  };

  testEnvComplex = {
    expr = complex.config.environment.etc;
    expected = {
      "nixsec/secrets/bar" = {
        mode = "0400";
        text = "bar";
      };
      "nixsec/secrets/foo" = {
        mode = "0400";
        text = "foo";
      };
    };
  };

  testConfigComplex = {
    expr = builtins.attrNames complex.options.secrets;
    expected = [
      "bar"
      "foo"
      "relevant_derivation_a"
      "relevant_derivation_b"
      "relevant_derivation_c"
    ];
  };

}
