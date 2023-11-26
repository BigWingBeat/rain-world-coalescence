using Menu;
using Menu.Remix.MixedUI;
using UnityEngine;

namespace MultiplayerMvpClient.Menu
{
	public unsafe class ServerLobbyMenu : SinglePageMenu
	{
		public const string DISCONNECT_BUTTON_SIGNAL = "DISCONNECT";

		public static readonly ProcessManager.ProcessID ProcessId = new(nameof(ServerLobbyMenu), true);

		private OpLabel UsernameDisplayLabel;

		private SafeAppHandle? appHandle;

		private ClientProfile Profile;

		// Fields are initialised in `CommunicateWithPreviousProcess`
#pragma warning disable CS8618
		public ServerLobbyMenu(ProcessManager manager, ProcessManager.ProcessID ID) : base(manager, ID)
		{
			new CustomMenuBuilder()
				.WithBackgroundArt(true)
				// .WithTitleIllustration("MultiplayerTitle")
				.WithStandardBackButton(DISCONNECT_BUTTON_SIGNAL)
				// .WithSong("RW_43 - Bio Engineering")
				.Build(this, Page);
		}
#pragma warning restore

		public override IEnumerable<MenuObject> YieldMenuObjects()
		{
			yield break;
		}

		public override IEnumerable<UIelement> YieldMixedUiElements()
		{
			Vector2 usernameDisplayAnchor = new(ScreenDimensions.ScreenCenter.x, ScreenDimensions.ScreenSize.y * 0.6f);
			UsernameDisplayLabel = new(
				usernameDisplayAnchor,
				 new(20, 24),
				  string.Empty,
				  FLabelAlignment.Center);

			yield return UsernameDisplayLabel;
		}

		internal static void SetupHooks()
		{
			SetupSwitchMainProcessHook(ProcessId, (manager, ID) => new ServerLobbyMenu(manager, ID));
		}

		public override void RawUpdate(float dt)
		{
			if (appHandle != null && !appHandle.IsInvalid && !appHandle.IsClosed)
			{
				bool exitRequested = appHandle.Update();
				if (exitRequested)
				{
					Plugin.Logger.LogInfo("Native app requested exit");
					Disconnect();
				}
			}

			base.RawUpdate(dt);
		}

		public void CommunicateWithPreviousProcess(SafeAppHandle? appHandle, ClientProfile profile)
		{
			this.appHandle = appHandle;
			Profile = profile;
			UsernameDisplayLabel.text = $"Connected to server as '{profile.Username}'";
		}

		private void Disconnect()
		{
			if (!IsSwitchingMainProcess)
			{
				appHandle?.Close();
				appHandle = null;
				PlaySound(SoundID.MENU_Switch_Page_Out);
				// manager.musicPlayer?.FadeOutAllSongs(100f);
				SwitchMainProcess(ServerBrowserMenu.ProcessId);
			}
		}

		public override void Singal(MenuObject sender, string message)
		{
			switch (message)
			{
				case DISCONNECT_BUTTON_SIGNAL:
					Disconnect();
					break;
			}
		}
	}
}
