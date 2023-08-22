using BepInEx;
using BepInEx.Logging;
using Menu;
using MultiplayerMvpClient.NativeInterop;
using MultiplayerMvpClient.Plugin.Menu;
using RWCustom;
using UnityEngine;
using VanillaMenu = Menu.Menu;

namespace MultiplayerMvpClient.Plugin
{
	[BepInPlugin(MyPluginInfo.PLUGIN_GUID, MyPluginInfo.PLUGIN_NAME, MyPluginInfo.PLUGIN_VERSION)]
	public class MultiplayerMvpClientPlugin : BaseUnityPlugin
	{
		public const string PLUGIN_GUID = MyPluginInfo.PLUGIN_GUID;
		public const string PLUGIN_NAME = MyPluginInfo.PLUGIN_NAME;
		public const string PLUGIN_VERSION = MyPluginInfo.PLUGIN_VERSION;

#pragma warning disable CS8618 // Statics get populated in the constructor
		public static MultiplayerMvpClientPlugin Instance { get; private set; }

		internal static new ManualLogSource Logger { get; private set; }
#pragma warning restore CS8618

		private MultiplayerMvpClientPlugin() : base()
		{
			Instance ??= this;
			Logger ??= base.Logger;
		}

#pragma warning disable IDE0051, CA1822 // Unity uses reflection to call Awake, for this to work it must not be static
		private void Awake()
		{
			SetupHooks();
			MultiplayerLobby.SetupHooks();
		}
#pragma warning restore IDE0051, CA1822

		private static void SetupHooks()
		{
			MultiplayerMvpNative.set_error_handler(DisplayNativeError);
			Application.quitting += DestroyStaticTaskPools;
		}

		internal static void DisplayNativeError(string text)
		{
			if (Custom.rainWorld?.processManager?.currentMainLoop is VanillaMenu menu && Custom.rainWorld?.options?.ScreenSize.x is float screenWidth)
			{
				const string NativeErrorDialogMessage = $"{PLUGIN_GUID}::NATIVE_ERROR_DIALOG";
				Vector2 standardDialogPosition = new((screenWidth / 2) - 240f + ((1366f - screenWidth) / 2f), 224f);
				Vector2 standardDialogSize = new(480f, 320f);
				DialogBoxNotify dialog = new(menu, menu.pages[0], text, NativeErrorDialogMessage, standardDialogPosition, standardDialogSize);
				menu.pages[0].subObjects.Add(dialog);

				On.Menu.MenuObject.hook_Singal dismissDialogEvent = null!;
				dismissDialogEvent = DismissDialog;
				On.Menu.MenuObject.Singal += dismissDialogEvent;

				void DismissDialog(On.Menu.MenuObject.orig_Singal orig, MenuObject self, MenuObject sender, string message)
				{
					if (message == NativeErrorDialogMessage && ReferenceEquals(sender, dialog.continueButton))
					{
						On.Menu.MenuObject.Singal -= dismissDialogEvent;
						menu.PlaySound(SoundID.MENU_Button_Standard_Button_Pressed);
						menu.pages[0].subObjects.Remove(dialog);
						dialog.RemoveSprites();
					}
					else
					{
						orig(self, sender, message);
					}
				}
			}
		}

		private static void DestroyStaticTaskPools()
		{
			MultiplayerMvpNative.destroy_static_taskpools();
		}
	}
}
