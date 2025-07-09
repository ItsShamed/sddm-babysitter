# Copyright (c) 2025 tsrk. <tsrk@tsrk.me>
# This file is licensed under the  license
# See the LICENSE file in the repository root for more info.

# SPDX-License-Identifier: MIT

{ self, ... }:

{ config, lib, pkgs, ... }:

let
  cfg = config.services.sddm-babysitter;
  sddm-babysitter = self.defaultPackage.${pkgs.system};
in
{
  key = ./module.nix;

  options = {
    services.sddm-babysitter = {
      enable = lib.options.mkEnableOption "a dameon to babysit SDDM if its helper dies";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [
      sddm-babysitter
    ];
    systemd.services.sddm-babysitter = {
      enable = true;
      after = [ "display-manager.service" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Type = "exec";
        ExecStart = "${sddm-babysitter}/bin/sddm-babysitter";
      };
    };
  };
}
