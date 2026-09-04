let
  hasLock = builtins.pathExists ./flake.lock;
  lock = if hasLock then builtins.fromJSON (builtins.readFile ./flake.lock) else { };

  # Deterministic, pinned flake-compat source resolution
  flakeCompatSrc =
    if hasLock && lock ? nodes && lock.nodes ? flake-compat
    then
      fetchTarball {
        url = "https://github.com/edolstra/flake-compat/archive/${lock.nodes.flake-compat.locked.rev}.tar.gz";
        sha256 = lock.nodes.flake-compat.locked.narHash;
      }
    else
      # Fixed cryptographic hash pinning to guarantee purity and reproducibility
      # when evaluating without flake.lock or in legacy nix-shell
      fetchTarball {
        url = "https://github.com/edolstra/flake-compat/archive/0f9255e01c2351bd7d402693824aab8d6d0ec37a.tar.gz";
        sha256 = "sha256-4L2oxhVuDQKtgp6pIFjy674UfEU8PuJHYW38KnZsmcc=";
      };

  flake = import flakeCompatSrc {
    src = ./.;
  };
in
flake.shellNix
