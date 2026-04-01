{
  nix.settings = {
    extra-substituters = [
      "s3://nix-cache-scaleway-sandbox-example?endpoint=s3.fr-par.scw.cloud&region=fr-par&scheme=https"
    ];
    extra-trusted-public-keys = [
      "nix-cache-sandbox:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
    ];
  };
}
