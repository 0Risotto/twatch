import { describe, expect, it } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import { useState } from 'react'
import { BlockedCategoriesEditor } from './BlockedCategoriesEditor'

function Harness() {
  const [value, setValue] = useState<string[]>([])
  return <BlockedCategoriesEditor value={value} onChange={setValue} />
}

describe('BlockedCategoriesEditor', () => {
  it('adds values with Enter and removes them', () => {
    render(<Harness />)
    const input = screen.getByLabelText('Add blocked category')

    fireEvent.change(input, { target: { value: 'blocked-tag' } })
    fireEvent.keyDown(input, { key: 'Enter' })
    expect(screen.getByText('blocked-tag')).toBeInTheDocument()

    fireEvent.click(screen.getByLabelText('Remove blocked-tag'))
    expect(screen.queryByText('blocked-tag')).not.toBeInTheDocument()
  })

  it('ignores duplicates and blank values', () => {
    render(<Harness />)
    const input = screen.getByLabelText('Add blocked category')

    fireEvent.change(input, { target: { value: ' ' } })
    fireEvent.keyDown(input, { key: 'Enter' })
    expect(screen.queryByRole('button', { name: /^Remove / })).not.toBeInTheDocument()

    fireEvent.change(input, { target: { value: '700-799' } })
    fireEvent.keyDown(input, { key: 'Enter' })
    fireEvent.change(input, { target: { value: '700-799' } })
    fireEvent.keyDown(input, { key: 'Enter' })
    expect(screen.getAllByText('700-799')).toHaveLength(1)
  })
})
