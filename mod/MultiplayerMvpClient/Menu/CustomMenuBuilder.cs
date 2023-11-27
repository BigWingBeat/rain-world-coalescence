using Menu;
using Menu.Remix;
using Menu.Remix.MixedUI;
using UnityEngine;

namespace MultiplayerMvpClient.Menu
{
	public class CustomMenuBuilder
	{
		private record BackgroundArtConfig(bool Dimmed, MenuScene.SceneID? SceneId);
		private BackgroundArtConfig? BackgroundArt;

		private record TitleIllustrationConfig(string FileName);
		private TitleIllustrationConfig? TitleIllustration;

		private record BackButtonConfig(string Signal, string? Label);
		private BackButtonConfig? BackButton;

		private record SongConfig(string Name, bool Loop, float Priority, float FadeInTime);
		private SongConfig? Song;

		public CustomMenuBuilder WithBackgroundArt(bool dimmed, MenuScene.SceneID? sceneId = null)
		{
			BackgroundArt = new(dimmed, sceneId);
			return this;
		}

		public CustomMenuBuilder WithTitleIllustration(string fileName)
		{
			TitleIllustration = new(fileName);
			return this;
		}

		public CustomMenuBuilder WithStandardBackButton(string signal = CustomMenuBase.BACK_BUTTON_SIGNAL, string? label = null)
		{
			BackButton = new(signal, label);
			return this;
		}

		public CustomMenuBuilder WithSong(string name, bool loop = true, float priority = 1f, float fadeInTime = 1f)
		{
			Song = new(name, loop, priority, fadeInTime);
			return this;
		}

		public void Build(CustomMenuBase menu, Page page)
		{
			if (BackgroundArt != null)
			{
				InteractiveMenuScene scene = new(menu, page, BackgroundArt.SceneId ?? (ModManager.MMF ? menu.manager.rainWorld.options.subBackground : MenuScene.SceneID.Landscape_SU));
				menu.scene = scene;
				page.subObjects.Add(scene);

				if (BackgroundArt.Dimmed)
				{
					FSprite backgroundTint = new("pixel")
					{
						color = Color.black,
						anchorX = 0,
						anchorY = 0,
						// Add an extra pixel of size on each side to avoid artifacts on the edge of the screen
						scaleX = menu.ScreenDimensions.ScreenSize.x + 2,
						scaleY = menu.ScreenDimensions.ScreenSize.y + 2,
						x = -1,
						y = -1,
						// Dimming is achieved by using a solid black colour with a partial alpha
						alpha = 0.85f
					};
					page.Container.AddChild(backgroundTint);
				}
			}

			if (TitleIllustration != null)
			{
				MenuIllustration title = new(menu, menu.scene, "", TitleIllustration.FileName, Vector2.zero, crispPixels: true, anchorCenter: false);
				title.sprite.shader = menu.manager.rainWorld.Shaders["MenuText"];
				menu.scene?.AddIllustration(title);
			}

			if (BackButton != null)
			{
				SimpleButton backButton = new(menu, page, BackButton.Label ?? menu.Translate(BackButton.Signal), BackButton.Signal, CustomMenuBase.StandardBackButtonPosition, CustomMenuBase.StandardButtonSize);
				page.subObjects.Add(backButton);
				menu.backObject = backButton;
			}

			foreach (MenuObject menuObject in menu.YieldMenuObjects())
			{
				page.subObjects.Add(menuObject);
			}

			MenuTabWrapper tabWrapper = new(menu, page);
			page.subObjects.Add(tabWrapper);

			foreach (UIelement element in menu.YieldMixedUiElements())
			{
				if (element is ICanBeTyped typeable)
				{
					menu.Typeables.Add(typeable);
				}

				// Required to actually add the UIelements to the menu
				_ = new UIelementWrapper(tabWrapper, element);
			}

			// If we're going to start playing our own song, fade out whatever may have been playing from the previous menu
			if (Song != null)
			{
				menu.manager.musicPlayer?.FadeOutAllSongs(25f);
				if (Song.Loop)
				{
					menu.Song = new(Song.Name, Song.Priority, Song.FadeInTime);
				}
				else
				{
					menu.manager.musicPlayer?.MenuRequestsSong(Song.Name, Song.Priority, Song.FadeInTime);
				}
			};
		}
	}
}
