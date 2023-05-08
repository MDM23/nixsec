{
  inputs = { runtimeLib = { url = "github:nix-community/nixpkgs.lib"; }; };

  outputs = { self, runtimeLib, ... }: {
    lib.mkSecrets = configFile:
      let nixsecConfig = import configFile;
      in { lib, pkgs, config, ... }:
      with lib;
      let
        # Checks if the given secret should be made available to the host that
        # is currently beeing configured.
        isRelevantSecret = n: v:
          builtins.elem config.networking.hostName v.hosts;

        # Returns a list of all secrets that should be mdae available to the
        # host that is currently beeing configured.
        relevantSecrets =
          attrsets.filterAttrs isRelevantSecret nixsecConfig.secrets;

        # Checks if the given derivation uses only secrets that are available
        # to the host that is currently beeing configured.
        isRelevantDerivation = n: v:
          builtins.all (sec: isRelevantSecret sec nixsecConfig.secrets."${sec}")
          (attrNames (functionArgs v));

        # Returns a list containing all derivations which only depend on
        # secrets that are relevant to the currently configured host.
        relevantDerivations =
          attrsets.filterAttrs isRelevantDerivation nixsecConfig.derivations;

        # Write a helper script that builds a given derived secret and writes
        # the result to stdout.
        deriveRuntime = pkgs.writeShellScript "derive.sh" ''
          ${pkgs.nix}/bin/nix eval --impure --raw -f - << EOF
            with builtins; let
              lib = import ${runtimeLib}/lib;
              nixsecConfig = import ${configFile};
              drv = nixsecConfig.derivations."$1";
            in
            drv
              (mapAttrs
                (n: v: (lib.strings.removeSuffix "\n" (readFile "/run/nixsec/secrets/\''${n}")))
                (functionArgs drv))
          EOF
        '';
      in {
        options.secrets = (attrsets.mapAttrs (n: v:
          mkOption {
            type = types.str;
            default = "/run/nixsec/secrets/${n}";
          }) relevantSecrets) // attrsets.mapAttrs (n: v:
            mkOption {
              type = types.str;
              default = "/run/nixsec/secrets/${n}";
            }) relevantDerivations;

        config.environment.etc = attrsets.mapAttrs' (n: v:
          nameValuePair "nixsec/secrets/${n}" {
            mode = "0400";
            text = v.enc;
          }) relevantSecrets;
        
        config.system.activationScripts.nixsec = ''
          mkdir -p "/run/nixsec/secrets"
          umask 277

          for file in /etc/nixsec/secrets/*; do
            [ -f "$file" ] || continue
            ${pkgs.age}/bin/age \
              -i /etc/ssh/ssh_host_ed25519_key \
              -o "/run/nixsec/secrets/$(basename "$file")" \
              -d "$file"
          done

          for drv in ${
            builtins.concatStringsSep " " (attrNames relevantDerivations)
          }; do
            target="/run/nixsec/secrets/$drv"
            [ -f "$target" ] && continue
            ${deriveRuntime} "$drv" > "$target"
          done
        '';
      };

    tests = let
      results = import ./tests {
        inherit self;
        lib = import "${runtimeLib}/lib";
      };
    in if results == [ ] then
      "All tests passed"
    else
      throw (builtins.toJSON results);
  };
}
