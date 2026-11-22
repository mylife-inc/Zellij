// Shared component identities: offline output hydrates the same controls as Next.
import { Analytics } from '../chrome/analytics';
import { GuideSidebar } from '../guide/sidebar';
import { TableOfContents } from '../guide/table-of-contents';
import { Mermaid } from '../mdx/mermaid';
import { TabsClient } from '../mdx/tabs-client';
import { CodeBlock } from '../mdx/code-block';
import { GradientReveal } from '../motion/gradient-reveal';
import { ScrollStage, Parallax } from '../motion/scroll-stage';
import { CountUp } from '../motion/count-up';
import { Reveal } from '../motion/reveal';
import { SearchModal } from '../search/search-modal';
import { NavBarClient } from '../chrome/nav-bar-client';
import { AnnouncementBar } from '../chrome/announcement-bar';
import { MobileDrawer } from '../chrome/mobile-drawer';
import { ThemeSwitcher } from '../chrome/theme-switcher';
import { ThemeToggle } from '../chrome/theme-toggle';
import { CardCarouselTrack } from '../sections/components/card-carousel-track';
import { GalleryLightbox } from '../sections/components/gallery-lightbox';
import { MediaAccordionControl } from '../sections/components/media-accordion-control';
import { StoryScroll } from '../sections/components/story-scroll';
import { VideoPlayer } from '../sections/components/video-player';
import { TestimonialCarousel } from '../sections/components/testimonial-carousel';
import { ChoiceControl } from '../sections/components/choice-control';
import { CopyCommand } from '../sections/components/copy-command';
import { ComposeScrollStage } from '../sections/components/compose-scroll';
import { ComposeBackdropMedia } from '../sections/components/compose-backdrop';
import { PanelVideo } from '../sections/components/panel-video';
export const components = { Analytics, GuideSidebar, TableOfContents, Mermaid, TabsClient, CodeBlock,
  GradientReveal, ScrollStage, Parallax, CountUp, Reveal, SearchModal, NavBarClient,
  AnnouncementBar, MobileDrawer, ThemeSwitcher, ThemeToggle, CardCarouselTrack,
  GalleryLightbox, MediaAccordionControl, StoryScroll, VideoPlayer, TestimonialCarousel,
  ChoiceControl, CopyCommand, ComposeScrollStage, ComposeBackdropMedia, PanelVideo };
