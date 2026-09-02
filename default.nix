let
  hasLock = builtins.pathExists ./flake.lock;
  lock = if hasLock then builtins.fromJSON (builtins.readFile ./flake.lock) else { };
  flakeCompatSrc =
    if hasLock && lock ? nodes && lock.nodes ? flake-compat
    then
      fetchTarball {
        url = "https://github.com/edolstra/flake-compat/archive/${lock.nodes.flake-compat.locked.rev}.tar.gz";
        sha256 = lock.nodes.flake-compat.locked.narHash;
      }
    else
      fetchTarball {
        url = "https://github.com/edolstra/flake-compat/archive/master.tar.gz";
      };
  flake = import flakeCompatSrc {
    src = ./.;
  };
in
flake.defaultNix
