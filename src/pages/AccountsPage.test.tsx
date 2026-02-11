import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { AccountsPage } from './AccountsPage'

describe('AccountsPage Component', () => {
  it('should render accounts page with header and list area', () => {
    render(<AccountsPage />)

    expect(screen.getByTestId('accounts-page')).toBeInTheDocument()
    expect(screen.getByTestId('accounts-header')).toBeInTheDocument()
    expect(screen.getByTestId('accounts-list')).toBeInTheDocument()
  })

  it('should render Add Account button in the header', () => {
    render(<AccountsPage />)

    const header = screen.getByTestId('accounts-header')
    const button = screen.getByTestId('add-account-button')

    expect(button).toBeInTheDocument()
    expect(button).toHaveTextContent('Add Account')
    expect(header).toContainElement(button)
  })

  it('should render header with button aligned to the right', () => {
    render(<AccountsPage />)

    const header = screen.getByTestId('accounts-header')
    expect(header).toHaveClass('flex')
    expect(header).toHaveClass('justify-end')
    expect(header).toHaveClass('items-center')
  })

  it('should not render any title text', () => {
    render(<AccountsPage />)

    expect(screen.queryByText('Accounts')).not.toBeInTheDocument()
    expect(screen.queryByText('Account management')).not.toBeInTheDocument()
  })

  it('should use flex column layout with header and scrollable list', () => {
    render(<AccountsPage />)

    const page = screen.getByTestId('accounts-page')
    expect(page).toHaveClass('flex', 'flex-col', 'h-full', 'p-6')

    const list = screen.getByTestId('accounts-list')
    expect(list).toHaveClass('flex-1')
  })

  it('should have proper page spacing and padding', () => {
    render(<AccountsPage />)

    const page = screen.getByTestId('accounts-page')
    expect(page).toHaveClass('p-6')

    const header = screen.getByTestId('accounts-header')
    expect(header).toHaveClass('mb-4')
  })

  it('should have styled list area with background', () => {
    render(<AccountsPage />)

    const list = screen.getByTestId('accounts-list')
    expect(list).toHaveClass('bg-neutral-900/50')
    expect(list).toHaveClass('rounded-lg')
    expect(list).toHaveClass('border')
  })

  it('should have proper button styling with red accent', () => {
    render(<AccountsPage />)

    const button = screen.getByTestId('add-account-button')
    expect(button).toHaveClass('bg-red-600')
    expect(button).toHaveClass('hover:bg-red-700')
    expect(button).toHaveClass('text-white')
    expect(button).toHaveClass('px-6')
    expect(button).toHaveClass('py-2.5')
    expect(button).toHaveClass('rounded-md')
  })
})
