import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Layout } from './Layout'
import type { NavigationItem } from '../types/layout'

describe('Layout Component', () => {
  const mockItems: NavigationItem[] = [
    { id: 'accounts', label: 'Accounts', icon: '' },
    { id: 'settings', label: 'Settings', icon: '' },
  ]

  it('should render layout with navigation and main window', () => {
    render(
      <Layout navigationItems={mockItems}>
        <div>Main Content</div>
      </Layout>
    )

    expect(screen.getByText('Accounts')).toBeInTheDocument()
    expect(screen.getByText('Settings')).toBeInTheDocument()
    expect(screen.getByText('Main Content')).toBeInTheDocument()
  })

  it('should render all navigation items', () => {
    render(
      <Layout navigationItems={mockItems}>
        <div>Content</div>
      </Layout>
    )

    mockItems.forEach((item) => {
      expect(screen.getByText(item.label)).toBeInTheDocument()
    })
  })

  it('should call onNavigate when a navigation item is clicked', async () => {
    const user = userEvent.setup()
    const handleNavigate = vi.fn()

    render(
      <Layout
        navigationItems={mockItems}
        onNavigate={handleNavigate}
      >
        <div>Content</div>
      </Layout>
    )

    const accountsButton = screen.getByRole('button', { name: /Accounts/i })
    await user.click(accountsButton)

    expect(handleNavigate).toHaveBeenCalledWith('accounts')
  })

  it('should highlight active navigation item', () => {
    render(
      <Layout navigationItems={mockItems} activeItemId="accounts">
        <div>Content</div>
      </Layout>
    )

    const accountsButton = screen.getByRole('button', { name: /Accounts/i })
    expect(accountsButton).toHaveClass('active')
  })

  it('should render a two-column layout with navigation on left', () => {
    const { container } = render(
      <Layout navigationItems={mockItems}>
        <div data-testid="main-content">Content</div>
      </Layout>
    )

    const layout = container.querySelector('[data-testid="layout-container"]')
    expect(layout).toHaveClass('flex')

    const nav = container.querySelector('[data-testid="navigation-sidebar"]')
    expect(nav).toBeInTheDocument()

    const main = container.querySelector('[data-testid="main-window"]')
    expect(main).toBeInTheDocument()
  })

  it('should render with monotone color scheme', () => {
    const { container } = render(
      <Layout navigationItems={mockItems}>
        <div>Content</div>
      </Layout>
    )

    const nav = container.querySelector('[data-testid="navigation-sidebar"]')
    expect(nav).toHaveClass('bg-neutral-950')
  })
})
