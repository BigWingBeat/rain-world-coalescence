using Menu;
using Menu.Remix;
using Menu.Remix.MixedUI;
using UnityEngine;
using VanillaMenu = Menu.Menu;

namespace MultiplayerMvpClient.Menu
{
	public abstract class CustomMenuBase : VanillaMenu
	{
		public const string BACK_BUTTON_SIGNAL = "BACK";

		public static readonly Vector2 StandardButtonSize = new(110f, 30f);

		public static readonly Vector2 StandardBackButtonPosition = new(200f, 50f);

		public List<ICanBeTyped> Typeables;

		public readonly ScreenDimensions ScreenDimensions;

		public record SongConfig(string Name, float Priority, float FadeInTime);
		public SongConfig? Song;

		public bool IsSwitchingMainProcess { get; protected set; }

		private bool HasDoneInitUpdate;

		// Required to properly initialise certain ui elements, or else they render incorrectly
		// Affects at least OpTextBox & OpUpDown, possibly others as well
		public override bool FreezeMenuFunctions => HasDoneInitUpdate && base.FreezeMenuFunctions;

		public CustomMenuBase(ProcessManager manager, ProcessManager.ProcessID ID) : base(manager, ID)
		{
			Typeables = new(0);
			ScreenDimensions = new(manager);
		}

		public static void SetupSwitchMainProcessHook(ProcessManager.ProcessID processId, Func<ProcessManager, ProcessManager.ProcessID, MainLoopProcess> constructor)
		{
			On.ProcessManager.PostSwitchMainProcess += (On.ProcessManager.orig_PostSwitchMainProcess orig, ProcessManager self, ProcessManager.ProcessID ID) =>
			{
				if (ID == processId)
				{
					MainLoopProcess newProcess;
					try
					{
						newProcess = constructor(self, ID);
					}
					catch (Exception e)
					{
						Plugin.Logger.LogError($"Error while constructing new process '{processId}': {e}");
						throw;
					}
					self.currentMainLoop = newProcess;
				}
				orig(self, ID);
			};
		}

		public static void SetupMainMenuButtonHook(int indexFromBottomOfList, ProcessManager.ProcessID id, string signal, string? label = null)
		{
			On.Menu.MainMenu.ctor += (On.Menu.MainMenu.orig_ctor orig, MainMenu self, ProcessManager manager, bool showRegionSpecificBkg) =>
			{
				orig(self, manager, showRegionSpecificBkg);

				float buttonWidth = MainMenu.GetButtonWidth(self.CurrLang);
				Vector2 pos = new(683f - (buttonWidth / 2f), 0f);
				Vector2 size = new(buttonWidth, 30f);
				self.AddMainMenuButton(new SimpleButton(self, self.pages[0], label ?? self.Translate(signal), signal, pos, size), () =>
				{
					self.manager.RequestMainProcessSwitch(id);
					self.PlaySound(SoundID.MENU_Switch_Page_In);
				}, indexFromBottomOfList);
			};
		}

		public override void Init()
		{
			base.Init();
			init = true;
			Update();
			HasDoneInitUpdate = true;
		}

		public override void Update()
		{
			base.Update();

			// Play the specified song on loop
			if (Song != null && manager.musicPlayer?.song == null)
			{
				manager.musicPlayer?.MenuRequestsSong(Song.Name, Song.Priority, Song.FadeInTime);
			}

			// Pressing Esc acts like clicking the back button
			if (backObject != null && RWInput.CheckPauseButton(0, manager.rainWorld))
			{
				Singal(backObject, BACK_BUTTON_SIGNAL);
			}
		}

		protected void BackOutMainProcessSwitch(ProcessManager.ProcessID id)
		{
			// Switching main process while a Dialog is up softlocks the game, so don't do that
			if (manager.dialog == null)
			{
				IsSwitchingMainProcess = true;
				PlaySound(SoundID.MENU_Switch_Page_Out);
				manager.musicPlayer?.FadeOutAllSongs(100f);
				manager.RequestMainProcessSwitch(id);
			}
		}

		public void DisplayNativeError(string text)
		{
			Plugin.Logger.LogError($"Error in native code: '{text}'");

			PlaySound(SoundID.MENU_Security_Button_Release);

			// Dialogs automatically disable most UI elements when shown, except for the typing aspects of ICanBeTyped implementors
			// These instead need to be manually disabled and reenabled when the dialog is shown and dismissed respectively
			// ICanBeTyped is implemented by OpComboBox, OpListBox, OpResourceList, OpResourceSelector, OpTextBox and OpUpdown
			foreach (ICanBeTyped typeable in Typeables)
			{
				typeable.Unassign();
			}
			DialogNotify dialog = new(text, manager, () =>
			{
				PlaySound(SoundID.MENU_Button_Standard_Button_Pressed);
				foreach (ICanBeTyped typeable in Typeables)
				{
					typeable.Assign();
				}
			});
			manager.ShowDialog(dialog);
		}
	}
}
